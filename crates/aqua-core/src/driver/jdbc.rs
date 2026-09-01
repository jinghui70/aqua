//! JDBC 驱动实现 - 通过 spawn connector.jar 访问 Oracle/信创数据库。
//!
//! 通信协议(architecture.md §2): 一次性命令,stdin JSON 请求 -> stdout JSON 响应 -> exit。
//! connector.jar (Java) 负责实际 JDBC 连接 + 反解,本驱动只做子进程通信。

use crate::driver::{ColumnMeta, DbConfig, Driver, DriverError, IndexMeta, TableInfo, TableMeta};
use crate::schema::DataType;
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::OnceCell;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows: spawn 子进程时不弹控制台黑窗口(GUI 进程 spawn java.exe 默认会弹黑窗)。
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 解析出的 java 可执行路径(进程生命周期内解析一次;None = 未找到显式路径,回退 PATH)。
static JAVA_BIN: OnceCell<Option<PathBuf>> = OnceCell::const_new();

/// 构造 java 子进程 Command(Windows 下禁用控制台黑窗口)。
///
/// 不直接依赖 PATH:GUI 启动的打包进程(macOS Finder/Dock、Windows 资源管理器)
/// 环境极简,PATH 里往往没有真 JDK(macOS 上还会命中 `/usr/bin/java` stub,
/// 输出 "No Java runtime present" 之类无法解析的内容)。先走 [resolve_java_path]
/// 显式解析,失败才回退 PATH(保持 `pnpm dev` 终端环境行为)。
async fn java_command() -> Command {
    let java = JAVA_BIN.get_or_init(resolve_java_path).await;
    #[allow(unused_mut)] // windows 下 creation_flags 需要 mut,其他平台不修改
    let mut cmd = match java {
        Some(path) => Command::new(path),
        None => Command::new("java"),
    };
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 报错排查用:当前实际使用的 java 路径描述。
async fn java_display() -> String {
    match JAVA_BIN.get_or_init(resolve_java_path).await {
        Some(path) => path.display().to_string(),
        None => "\"java\"(PATH 查找)".to_string(),
    }
}

/// 按优先级解析 java 可执行文件绝对路径:
///
/// 1. `JAVA_HOME/bin/java`(用户显式配置,三平台通用;报错文案一直承诺它,这里补上)
/// 2. 平台标准安装位置(macOS: `/usr/libexec/java_home` + homebrew;Windows:
///    Program Files 下常见发行版目录;Linux: `/usr/lib/jvm/*`)
/// 3. 都没有 -> None,由调用方回退 PATH
async fn resolve_java_path() -> Option<PathBuf> {
    if let Some(p) = java_from_env_home() {
        return Some(p);
    }
    platform_java_candidates()
        .await
        .into_iter()
        .find(|p| p.is_file())
}

fn java_exe_name() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

fn java_from_env_home() -> Option<PathBuf> {
    let home = std::env::var("JAVA_HOME").ok()?;
    let candidate = Path::new(home.trim()).join("bin").join(java_exe_name());
    candidate.is_file().then_some(candidate)
}

/// 平台标准安装位置候选,按优先级排列(同族目录内新版本优先)。
async fn platform_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // macOS 官方机制:返回已注册到 /Library/Java/JavaVirtualMachines 的最新 JDK Home。
        // 纯 homebrew 安装的 JDK 不注册,靠下面的 homebrew 目录枚举兜底。
        if let Some(home) = macos_java_home().await {
            candidates.push(home.join("bin").join("java"));
        }
        // Apple 的 /usr/bin/java stub 在 GUI 环境下不可靠,显式列 homebrew 路径。
        for base in ["/opt/homebrew/opt", "/usr/local/opt"] {
            for name in newest_first_subdirs(base, "openjdk") {
                candidates.extend(homebrew_java_paths(&Path::new(base).join(name)));
            }
        }
        for p in ["/opt/homebrew/bin/java", "/usr/local/bin/java"] {
            candidates.push(PathBuf::from(p));
        }
        // 兜底:登录 shell 的 PATH 复刻(GUI 进程不加载 .zprofile/.zshrc,
        // 终端能找到的 sdkman/jenv/asdf 等自定义位置只有这里能覆盖)。
        if let Some(p) = java_from_login_shell().await {
            candidates.push(p);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 发行版 JDK 惯例:/usr/lib/jvm/java-21-openjdk-amd64 等
        for name in newest_first_subdirs("/usr/lib/jvm", "") {
            candidates.push(Path::new("/usr/lib/jvm").join(name).join("bin").join("java"));
        }
    }

    #[cfg(windows)]
    {
        // 常见发行版默认安装根目录(Oracle / Temurin / Microsoft / Zulu / Corretto / Liberica),
        // 每个发行版目录内取最新版本
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
        for vendor in [
            "Java",
            "Eclipse Adoptium",
            "Microsoft",
            "Zulu",
            "Amazon Corretto",
            r"BellSoft\Liberica",
        ] {
            let vendor_dir = Path::new(&program_files).join(vendor);
            for name in newest_first_subdirs(&vendor_dir.to_string_lossy(), "") {
                candidates.push(vendor_dir.join(name).join("bin").join("java.exe"));
            }
        }
    }

    // 平台无候选(如非上述系统)时返回空,调用方回退 PATH
    candidates
}

/// homebrew openjdk formula 的 java 路径(按优先级)。
///
/// 版本化 formula(openjdk@21 等)是 keg-only,真 JDK Home 嵌在
/// `libexec/openjdk.jdk/Contents/Home/`(brew 也不给 bin/ 建完整链接);
/// 主 formula(openjdk)则直接在 bin/ 下。两种布局都试。
fn homebrew_java_paths(formula_dir: &Path) -> Vec<PathBuf> {
    vec![
        formula_dir
            .join("libexec/openjdk.jdk/Contents/Home/bin/java"),
        formula_dir.join("bin/java"),
    ]
}

/// 用登录 shell 解析 java(GUI 进程的通用兜底)。
///
/// GUI 启动的进程不加载 `.zprofile`/`.zshrc`,PATH 与终端不同——这正是
/// "dev 能用、打包版找不到"的根因。`$SHELL -lic` 会加载登录+交互配置,
/// `command -v java` 等价于终端里的 which,覆盖 sdkman/jenv/asdf 等任意
/// 自定义安装位置。带超时,失败静默返回 None 走后续候选。
async fn java_from_login_shell() -> Option<PathBuf> {
    let shell = std::env::var("SHELL").ok()?;
    let output = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new(&shell)
            .arg("-lic")
            .arg("command -v java")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await
    .ok()?
    .ok()?;

    if !output.status.success() {
        return None;
    }
    // shell 配置可能往 stdout 打噪音(nvm 等);只认绝对路径行,从后往前取
    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .rev()
        .map(|l| l.trim())
        .filter(|l| l.starts_with('/'))
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

/// 执行 `/usr/libexec/java_home`,返回最新注册 JDK Home(带超时,失败返回 None)。
async fn macos_java_home() -> Option<PathBuf> {
    let output = tokio::time::timeout(
        Duration::from_secs(3),
        tokio::process::Command::new("/usr/libexec/java_home")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output(),
    )
    .await
    .ok()?
    .ok()?;

    if !output.status.success() {
        return None;
    }
    let home = String::from_utf8(output.stdout).ok()?;
    let home = home.trim();
    (!home.is_empty()).then(|| PathBuf::from(home))
}

/// 枚举 base 下名字以 prefix 开头的子目录,按版本号新 -> 旧排序返回目录名。
/// 目录不存在/不可读返回空。
fn newest_first_subdirs(base: &str, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(base) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| prefix.is_empty() || n.starts_with(prefix))
            .collect(),
        Err(_) => Vec::new(),
    };
    sort_newest_first(&mut names);
    names
}

/// 目录名按版本号新 -> 旧排序(原地)。
fn sort_newest_first(names: &mut [String]) {
    names.sort_by_key(|n| std::cmp::Reverse(version_key(n)));
}

/// 提取名字中的数字序列作为版本比较键:"jdk-17.0.2" -> [17, 0, 2]。
/// 逐段比较使 21 排在 17 前、17 排在 1.8 前。
fn version_key(name: &str) -> Vec<u64> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// 连接 Java 数据源所需的最低 JDK 版本。
const MIN_JAVA_MAJOR: u32 = 17;

/// JDBC 驱动(spawn connector.jar)。
pub struct JdbcDriver {
    config: DbConfig,
    connector_path: String,
    /// drivers/ 目录(含 databases.json + 外置 JDBC jar)。
    /// 传给 connector,触发其加载 installed 驱动(Oracle 等)。
    drivers_dir: Option<PathBuf>,
    /// java 版本检测缓存(首次 call 时检测一次,通过则后续跳过,避免每次 spawn 开销)。
    java_checked: OnceCell<()>,
}

impl JdbcDriver {
    /// 创建 JDBC 驱动。
    ///
    /// - `connector_path`: connector.jar 路径(打包后为 resource_dir 绝对路径)。
    /// - `drivers_dir`: drivers/ 目录;`Some` 时 connector 会加载其中 installed 的 JDBC jar。
    pub fn new(config: &DbConfig, connector_path: &str, drivers_dir: Option<PathBuf>) -> Self {
        Self {
            config: config.clone(),
            connector_path: connector_path.to_string(),
            drivers_dir,
            java_checked: OnceCell::new(),
        }
    }

    /// 检测 java 运行时是否存在且版本 >= 17(首次检测后缓存结果)。
    ///
    /// 在每次 `call` 前调用:缺失/版本不足时返回清晰提示,不静默失败。
    /// 失败不缓存,用户安装/升级 JDK 后重试会重新检测。
    async fn check_java(&self) -> Result<(), DriverError> {
        self.java_checked
            .get_or_try_init(|| async { check_java_once().await })
            .await
            .map(|_| ())
    }

    /// 调用 connector.jar,发送请求,返回响应 JSON。
    async fn call(&self, action: &str, extra: Option<Value>) -> Result<Value, DriverError> {
        // 连接前确保 java >= 17(首次检测后缓存)
        self.check_java().await?;

        let mut request = json!({
            "action": action,
            "dialect": self.config.dialect,
            "host": self.config.host,
            "port": self.config.port,
            "user": self.config.user,
            "password": self.config.password,
            "database": self.config.database,
        });

        // Windows: Tauri resource_dir/app_data_dir 返回带 `\\?\` verbatim 前缀的路径
        // (绕过 MAX_PATH 限制)。但 Java launcher 无法打开带此前缀的 jar,报
        // "尝试打开文件 \\?\... 时出现意外错误"。strip 成普通 Win32 路径再传给 java。
        // 其他平台路径无此前缀,strip 为 no-op。
        let connector_path = strip_verbatim(&self.connector_path).to_string();
        let drivers_dir = self
            .drivers_dir
            .as_ref()
            .map(|d| strip_verbatim(&d.to_string_lossy()).to_string());

        // 传 drivers/ 目录,connector 据此加载 installed 的外置 JDBC jar(Oracle 等)
        if let Some(ref dir) = drivers_dir {
            request["driversDir"] = json!(dir);
        }

        if let Some(extra_val) = extra {
            if let (Some(req_map), Some(extra_map)) =
                (request.as_object_mut(), extra_val.as_object())
            {
                req_map.extend(extra_map.clone());
            }
        }

        let mut cmd = java_command().await;
        cmd.arg("-jar")
            .arg(&connector_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| {
                log::error!("spawn connector 失败: {}", e);
                DriverError::ConnectionFailed(format!("启动 connector 失败(需 JDK 17+): {}", e))
            })?;

        // 写 stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(request.to_string().as_bytes())
                .await
                .map_err(|e| DriverError::ConnectionFailed(format!("写 stdin 失败: {}", e)))?;
        }

        // 读 stdout
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| DriverError::ConnectionFailed(format!("connector 执行失败: {}", e)))?;

        // 错误处理:优先解析 stdout 的 JSON 业务错误(Java 经 writeError 写 stdout);
        // stdout 非 JSON 时(通常是 JVM launcher 直接报错,如"无法访问 jarfile"),
        // 按 GBK 解码回传,避免中文乱码遮蔽真实原因。
        let stdout_str = decode_console(&output.stdout);
        let stderr_str = decode_console(&output.stderr);

        // stdout 可能为有效 JSON 响应(含 {error:...}),即使 exit≠0 也优先取
        if let Ok(response) = serde_json::from_slice::<Value>(&output.stdout) {
            if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
                return Err(DriverError::QueryFailed(error.to_string()));
            }
        }

        if !output.status.success() {
            // JVM launcher 报错或进程异常:stdout+stderr 合并,按系统编码解码可读
            let detail = if !stdout_str.trim().is_empty() {
                &stdout_str
            } else {
                &stderr_str
            };
            log::error!(
                "connector 失败 exit={}: stdout={} stderr={}",
                output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into()),
                stdout_str.trim(),
                stderr_str.trim()
            );
            return Err(DriverError::ConnectionFailed(format!(
                "connector 失败(exit={}): {}",
                output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into()),
                detail.trim()
            )));
        }

        let response: Value = serde_json::from_slice(&output.stdout).map_err(|e| {
            log::error!("connector 响应解析失败: {} stdout={}", e, stdout_str);
            DriverError::ConnectionFailed(format!("connector 响应解析失败: {}", e))
        })?;

        Ok(response)
    }
}

/// 去掉 Windows verbatim 路径前缀 `\\?\`。
///
/// Tauri 在 Windows 上 `resource_dir`/`app_data_dir` 返回带 `\\?\` 前缀的 verbatim 路径
/// (用于绕过 MAX_PATH 260 限制)。但 Java launcher 无法打开带此前缀的 jar,报
/// "尝试打开文件 \\?\... 时出现意外错误"。strip 成普通 Win32 路径后 java 可正常打开。
/// connector.jar 在本地盘且路径远 < 260,strip 安全;其他平台无此前缀,no-op。
fn strip_verbatim(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

/// 解码子进程输出:先严格 UTF-8,失败回退 GBK(Windows 中文控制台)。
///
/// connector 正常 JSON 响应为 UTF-8/ASCII,严格解码必成功;
/// JVM launcher 的本地化错误(如"无法访问 jarfile")在中文 Windows 上按 GBK 输出,
/// UTF-8 严格解码失败 -> 回退 GBK -> 可读。跨平台自适应,无需平台条件编译。
fn decode_console(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => encoding_rs::GBK.decode(bytes).0.into_owned(),
    }
}

/// 检测 java 运行时:spawn `java -version`,解析主版本号,要求 >= 17。
///
/// `java -version` 将版本信息输出到 stderr。缺失或版本不足返回带明确指引的 `DriverError`。
async fn check_java_once() -> Result<(), DriverError> {
    let java_desc = java_display().await;
    let mut cmd = java_command().await;
    cmd.arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd
        .output()
        .await
        .map_err(|e| {
            log::error!("spawn java -version 失败({}): {}", java_desc, e);
            DriverError::ConnectionFailed(format!(
                "未检测到 Java 运行时(连接 JDBC 数据源需 JDK 17+,请安装并配置 JAVA_HOME/PATH)。尝试执行: {}: {}",
                java_desc, e
            ))
        })?;

    // java -version 输出到 stderr;个别发行版可能输出到 stdout,合并解析
    let combined = format!(
        "{}\n{}",
        decode_console(&output.stderr),
        decode_console(&output.stdout)
    );

    let major = parse_java_major_version(&combined).ok_or_else(|| {
        // 典型场景:GUI 环境命中 macOS /usr/bin/java stub,输出 "No Java runtime present"
        log::warn!("无法解析 Java 版本(java: {}),原始输出: {}", java_desc, combined);
        DriverError::ConnectionFailed(format!(
            "无法解析 Java 版本(连接 JDBC 数据源需 JDK 17+,请检查 JAVA_HOME/PATH 配置)。java: {}, java -version 输出: {}",
            java_desc,
            combined.trim()
        ))
    })?;

    if major < MIN_JAVA_MAJOR {
        return Err(DriverError::ConnectionFailed(format!(
            "Java 版本过低(当前 {},需 {}+),请升级 JDK 并配置 JAVA_HOME/PATH",
            major, MIN_JAVA_MAJOR
        )));
    }

    Ok(())
}

/// 从 `java -version` 输出解析主版本号。
///
/// - Java 8 及更早:`1.8.0_292` -> 8
/// - Java 9+:`17.0.1` -> 17、`21.0.1` -> 21
fn parse_java_major_version(output: &str) -> Option<u32> {
    let line = output.lines().find(|l| l.contains("version"))?;
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    let version_str = &rest[..end]; // e.g. "17.0.1" / "1.8.0_292"

    let mut parts = version_str.split('.');
    let first = parts.next()?;
    // 旧版 "1.8.x" 取第二段;新版 "17.x" 取首段
    let major_str = if first == "1" {
        parts.next()?.split('_').next()?
    } else {
        first
    };
    major_str.parse().ok()
}

#[async_trait]
impl Driver for JdbcDriver {
    async fn test_connection(&self) -> Result<(), DriverError> {
        self.call("testConnection", None).await?;
        Ok(())
    }

    async fn list_tables(&self, schema: &str) -> Result<Vec<TableInfo>, DriverError> {
        let resp = self
            .call("listTables", Some(json!({ "schema": schema })))
            .await?;

        let tables = resp
            .get("tables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        let name = v.get("name")?.as_str()?.to_string();
                        let comment = v
                            .get("comment")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string());
                        Some(TableInfo { name, comment })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tables)
    }

    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, DriverError> {
        let resp = self
            .call("getColumns", Some(json!({ "table": table })))
            .await?;

        let columns = resp
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_column_meta).collect())
            .unwrap_or_default();

        Ok(columns)
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>, DriverError> {
        let resp = self
            .call("listIndexes", Some(json!({ "table": table })))
            .await?;

        let indexes = resp
            .get("indexes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_index_meta).collect())
            .unwrap_or_default();

        Ok(indexes)
    }

    async fn import_tables(&self, tables: &[String]) -> Result<Vec<TableMeta>, DriverError> {
        // 批量:一次 spawn 反解全部选中表,替代逐表 2N 次 JVM 冷启动。
        let resp = self
            .call("importTables", Some(json!({ "tables": tables })))
            .await?;

        let metas = resp
            .get("tables")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| {
                        let name = t.get("name")?.as_str()?.to_string();
                        let columns = t
                            .get("columns")
                            .and_then(|v| v.as_array())
                            .map(|a| a.iter().filter_map(parse_column_meta).collect())
                            .unwrap_or_default();
                        let indexes = t
                            .get("indexes")
                            .and_then(|v| v.as_array())
                            .map(|a| a.iter().filter_map(parse_index_meta).collect())
                            .unwrap_or_default();
                        Some(TableMeta {
                            name,
                            columns,
                            indexes,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(metas)
    }

    async fn query_table_rows(&self, table: &str) -> Result<Vec<Map<String, Value>>, DriverError> {
        let resp = self.call("queryRows", Some(json!({ "table": table }))).await?;
        let empty = Vec::new();
        let rows = resp.get("rows").and_then(|v| v.as_array()).unwrap_or(&empty);
        // rows: [[v1, v2, ...], ...]; columns 从 resp.columns 取
        let columns: Vec<String> = resp
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|c| c.as_str().map(|s| s.to_uppercase())).collect())
            .unwrap_or_default();
        let mut result = Vec::new();
        for row in rows {
            if let Some(arr) = row.as_array() {
                let mut map = Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let val = arr.get(i).cloned().unwrap_or(Value::Null);
                    map.insert(col.clone(), val);
                }
                result.push(map);
            }
        }
        Ok(result)
    }

    async fn execute_update(&self, sql: &str) -> Result<usize, DriverError> {
        let resp = self.call("executeUpdate", Some(json!({ "sql": sql }))).await?;
        let affected = resp.get("affected").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        Ok(affected)
    }
}

/// 解析 connector 返回的列元数据。
fn parse_column_meta(v: &Value) -> Option<ColumnMeta> {
    Some(ColumnMeta {
        name: v.get("name")?.as_str()?.to_string(),
        data_type: parse_data_type(v.get("dataType")?.as_str()?),
        length: v.get("length").and_then(|x| x.as_u64()).map(|n| n as u32),
        precision: v
            .get("precision")
            .and_then(|x| x.as_u64())
            .map(|n| n as u32),
        scale: v.get("scale").and_then(|x| x.as_u64()).map(|n| n as u32),
        nullable: v.get("nullable").and_then(|x| x.as_bool()).unwrap_or(true),
        is_key: v.get("isKey").and_then(|x| x.as_bool()).unwrap_or(false),
        default_value: v
            .get("defaultValue")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        comment: v
            .get("comment")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
    })
}

/// 解析 connector 返回的索引元数据。
fn parse_index_meta(v: &Value) -> Option<IndexMeta> {
    Some(IndexMeta {
        name: v.get("name")?.as_str()?.to_string(),
        fields: v
            .get("fields")?
            .as_array()?
            .iter()
            .filter_map(|f| f.as_str().map(|s| s.to_string()))
            .collect(),
        unique: v.get("unique").and_then(|x| x.as_bool()).unwrap_or(false),
    })
}

/// connector 返回的逻辑类型字符串 -> DataType。
fn parse_data_type(s: &str) -> DataType {
    match s.to_uppercase().as_str() {
        "VARCHAR" => DataType::Varchar,
        "CLOB" => DataType::Clob,
        "TINYINT" => DataType::Tinyint,
        "INT" => DataType::Int,
        "LONG" => DataType::Long,
        "DECIMAL" => DataType::Decimal,
        "DOUBLE" => DataType::Double,
        "DATE" => DataType::Date,
        "DATETIME" => DataType::Datetime,
        "BLOB" => DataType::Blob,
        _ => DataType::Varchar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_type() {
        assert_eq!(parse_data_type("VARCHAR"), DataType::Varchar);
        assert_eq!(parse_data_type("CLOB"), DataType::Clob);
        assert_eq!(parse_data_type("INT"), DataType::Int);
        assert_eq!(parse_data_type("LONG"), DataType::Long);
        assert_eq!(parse_data_type("DOUBLE"), DataType::Double);
        assert_eq!(parse_data_type("DECIMAL"), DataType::Decimal);
        assert_eq!(parse_data_type("DATETIME"), DataType::Datetime);
        assert_eq!(parse_data_type("BLOB"), DataType::Blob);
    }

    #[test]
    fn test_parse_java_major_version() {
        // Java 9+: 主版本号取首段
        let openjdk17 = "openjdk version \"17.0.19\" 2026-04-21\n\
             OpenJDK Runtime Environment (build 17.0.19+0)";
        assert_eq!(parse_java_major_version(openjdk17), Some(17));

        let temurin21 = "openjdk version \"21.0.3\" 2024-04-16";
        assert_eq!(parse_java_major_version(temurin21), Some(21));

        // Java 8: "1.8.0_292" -> 8
        let java8 = "java version \"1.8.0_292\"\nJava(TM) SE Runtime Environment (build 1.8.0_292-b10)";
        assert_eq!(parse_java_major_version(java8), Some(8));

        // 版本输出在 stderr(实际场景),解析逻辑与位置无关
        assert_eq!(parse_java_major_version("noise\nversion \"11.0.1\""), Some(11));

        // 无法解析
        assert_eq!(parse_java_major_version("no version here"), None);
    }

    #[test]
    fn test_version_key() {
        assert_eq!(version_key("jdk-17.0.2"), vec![17, 0, 2]);
        assert_eq!(version_key("openjdk@21"), vec![21]);
        assert_eq!(version_key("java-1.8.0-openjdk"), vec![1, 8, 0]);
        assert_eq!(version_key("no-digits"), Vec::<u64>::new());
    }

    #[test]
    fn test_sort_newest_first() {
        let mut names = vec![
            "java-1.8.0-openjdk".to_string(),
            "java-21-openjdk".to_string(),
            "java-17-openjdk".to_string(),
        ];
        sort_newest_first(&mut names);
        // 新版本优先:21 > 17 > 1.8
        assert_eq!(
            names,
            vec!["java-21-openjdk", "java-17-openjdk", "java-1.8.0-openjdk"]
        );

        // homebrew 命名:openjdk@17 / openjdk@21
        let mut brew = vec!["openjdk@17".to_string(), "openjdk@21".to_string()];
        sort_newest_first(&mut brew);
        assert_eq!(brew, vec!["openjdk@21", "openjdk@17"]);
    }

    #[test]
    fn test_newest_first_subdirs_missing_dir() {
        // 目录不存在 -> 空(不 panic)
        assert!(newest_first_subdirs("/nonexistent/path/for/aqua-test", "openjdk").is_empty());
    }

    #[test]
    fn test_homebrew_java_paths_both_layouts() {
        let formula = Path::new("/opt/homebrew/opt/openjdk@21");
        let paths = homebrew_java_paths(formula);
        // 版本化 formula(keg-only)的 libexec 布局优先,主 formula 的 bin/ 布局其次
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home/bin/java"),
                PathBuf::from("/opt/homebrew/opt/openjdk@21/bin/java"),
            ]
        );
    }

    #[test]
    fn test_decode_console_utf8() {
        // 纯 ASCII / UTF-8:严格解码直通
        assert_eq!(decode_console(b"{\"status\":\"ok\"}"), "{\"status\":\"ok\"}");
        // UTF-8 中文(connector 正常 JSON 响应)
        assert_eq!(decode_console("连接成功".as_bytes()), "连接成功");
    }

    #[test]
    fn test_decode_console_gbk_fallback() {
        // GBK 编码的"错误"(JVM launcher 在中文 Windows 的输出)
        // "错误" 的 GBK 字节: 0xB4 0xED 0xCE 0xF3
        let gbk_bytes = &[0xB4, 0xED, 0xCE, 0xF3];
        assert_eq!(decode_console(gbk_bytes), "错误");
    }

    #[test]
    fn test_strip_verbatim() {
        // Windows verbatim 前缀 -> 普通 Win32 路径
        assert_eq!(
            strip_verbatim(r"\\?\C:\Users\app\connector.jar"),
            r"C:\Users\app\connector.jar"
        );
        // 无前缀 -> no-op
        assert_eq!(
            strip_verbatim(r"C:\Users\app\connector.jar"),
            r"C:\Users\app\connector.jar"
        );
        // 其他平台路径无此前缀 -> no-op
        assert_eq!(
            strip_verbatim("/usr/local/app/connector.jar"),
            "/usr/local/app/connector.jar"
        );
    }

    #[test]
    fn test_parse_column_meta() {
        let v = json!({
            "name": "USER_NAME",
            "dataType": "VARCHAR",
            "length": 64,
            "nullable": false,
            "isKey": false,
            "comment": "用户名"
        });

        let col = parse_column_meta(&v).unwrap();
        assert_eq!(col.name, "USER_NAME");
        assert_eq!(col.data_type, DataType::Varchar);
        assert_eq!(col.length, Some(64));
        assert!(!col.nullable);
        assert_eq!(col.comment, Some("用户名".to_string()));
    }

    #[test]
    fn test_parse_index_meta() {
        let v = json!({
            "name": "IDX_USER_NAME",
            "fields": ["USER_NAME", "STATUS"],
            "unique": true
        });

        let idx = parse_index_meta(&v).unwrap();
        assert_eq!(idx.name, "IDX_USER_NAME");
        assert_eq!(idx.fields, vec!["USER_NAME", "STATUS"]);
        assert!(idx.unique);
    }
}
