//! JDBC 驱动实现 - 通过 spawn connector.jar 访问 Oracle/信创数据库。
//!
//! 通信协议(architecture.md §2): 一次性命令,stdin JSON 请求 -> stdout JSON 响应 -> exit。
//! connector.jar (Java) 负责实际 JDBC 连接 + 反解,本驱动只做子进程通信。

use crate::driver::{ColumnMeta, DbConfig, Driver, DriverError, IndexMeta, TableInfo};
use crate::schema::DataType;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::OnceCell;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows: spawn 子进程时不弹控制台黑窗口(GUI 进程 spawn java.exe 默认会弹黑窗)。
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 构造 java 子进程 Command(Windows 下禁用控制台黑窗口)。
fn java_command() -> Command {
    #[allow(unused_mut)] // windows 下 creation_flags 需要 mut,其他平台不修改
    let mut cmd = Command::new("java");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
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

        // 诊断日志:完整 spawn 现场(password 脱敏),定位"手动成功、应用失败"差异
        log::info!(
            "spawn connector: java -jar {} (drivers_dir={:?})",
            connector_path,
            drivers_dir
        );
        log::info!("connector request: {}", redact_password(&request));

        let mut cmd = java_command();
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

        // 诊断日志:exit code + stdout/stderr 双解码(UTF-8 lossy 与 GBK),
        // 用于定位"手动成功、应用失败"的差异(Windows 中文控制台按 GBK 输出)。
        log::info!("connector exit: {:?}", output.status.code());
        log::info!("connector stdout (utf8-lossy): {}", String::from_utf8_lossy(&output.stdout));
        log::info!("connector stdout (gbk): {}", encoding_rs::GBK.decode(&output.stdout).0);
        log::info!("connector stderr (utf8-lossy): {}", String::from_utf8_lossy(&output.stderr));
        log::info!("connector stderr (gbk): {}", encoding_rs::GBK.decode(&output.stderr).0);

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
                stdout_str
            } else {
                stderr_str
            };
            return Err(DriverError::ConnectionFailed(format!(
                "connector 失败(exit={}): {}",
                output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into()),
                detail.trim()
            )));
        }

        let response: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| DriverError::ConnectionFailed(format!("connector 响应解析失败: {}", e)))?;

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

/// 生成 request JSON 的脱敏字符串(日志用):password 替换为 "***"。
fn redact_password(request: &Value) -> String {
    let mut sanitized = request.clone();
    if let Some(obj) = sanitized.as_object_mut() {
        if obj.contains_key("password") {
            obj.insert("password".to_string(), json!("***"));
        }
    }
    sanitized.to_string()
}

/// 检测 java 运行时:spawn `java -version`,解析主版本号,要求 >= 17。
///
/// `java -version` 将版本信息输出到 stderr。缺失或版本不足返回带明确指引的 `DriverError`。
async fn check_java_once() -> Result<(), DriverError> {
    let mut cmd = java_command();
    cmd.arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = cmd
        .output()
        .await
        .map_err(|e| {
            log::error!("spawn java -version 失败: {}", e);
            DriverError::ConnectionFailed(format!(
                "未检测到 Java 运行时(连接 JDBC 数据源需 JDK 17+,请安装并配置 JAVA_HOME/PATH): {}",
                e
            ))
        })?;

    // java -version 输出到 stderr;个别发行版可能输出到 stdout,合并解析
    let combined = format!(
        "{}\n{}",
        decode_console(&output.stderr),
        decode_console(&output.stdout)
    );

    let major = parse_java_major_version(&combined).ok_or_else(|| {
        log::warn!("无法解析 Java 版本,原始输出: {}", combined);
        DriverError::ConnectionFailed(
            "无法解析 Java 版本(连接 JDBC 数据源需 JDK 17+,请检查 JAVA_HOME/PATH 配置)".to_string(),
        )
    })?;

    if major < MIN_JAVA_MAJOR {
        return Err(DriverError::ConnectionFailed(format!(
            "Java 版本过低(当前 {},需 {}+),请升级 JDK 并配置 JAVA_HOME/PATH",
            major, MIN_JAVA_MAJOR
        )));
    }

    log::info!("java 检测通过,主版本 {}", major);
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
    fn test_redact_password() {
        let req = json!({ "action": "testConnection", "user": "admin", "password": "secret123" });
        let redacted = redact_password(&req);
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("admin")); // 非敏感字段保留
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
