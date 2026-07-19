# Design — connector spawn 诊断日志 + 编码修复

## 分层原则

- `aqua-core`:纯逻辑核心,**不依赖 tauri**。日志用 `log` facade(`log::info!/warn!/error!`)只 emit,不关心 sink。编码解码在此层(因为读字节在此层)。
- `src-tauri`:外壳装日志 sink(`tauri-plugin-log`),把 `log` facade 输出落到文件。

## 依赖新增

- `crates/aqua-core/Cargo.toml`:
  - `log = "0.4"`(facade)
  - `encoding_rs = "0.8"`(GBK/GB18030 解码,rust 生态标准)
- `src-tauri/Cargo.toml`:
  - `tauri-plugin-log = "2"`(cargo add 解析具体版本;它实现 `log` backend,自动捕获 aqua-core 的 emit)

## 日志落盘

`src-tauri/src/lib.rs::run()` 注册:

```rust
.plugin(
    tauri_plugin_log::Builder::new()
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir { file_name: Some("aqua".into()) },
        ))
        .level(log::LevelFilter::Info)
        .build(),
)
```

- Windows 落盘路径:`%LOCALAPPDATA%\com.aqua.app\logs\aqua.log`(identifier=com.aqua.app)。
- 需在 journal 记录该路径,交付时告诉用户去哪捞日志。
- CLI 模式(`cli.rs`)不 spawn java、也不初始化 tauri plugin;`log` facade 无 sink 时 emit 是 no-op,不报错,无需特殊处理。

## 编码解码策略(核心决策)

**先 UTF-8 严格解码,失败回退 GBK**,不用 `#[cfg]` 平台分支:

- connector 正常 JSON 响应:UTF-8/ASCII → 严格 UTF-8 必成功。
- JVM launcher 的 GBK 中文错误("错误: 无法访问 jarfile ..."):UTF-8 严格解码失败 → 回退 `encoding_rs::GBK` → 可读。
- 跨平台自适应(mac/linux 本就 UTF-8,走第一条),精准命中乱码场景,零平台条件编译。

新增辅助函数(jdbc.rs 私有):

```rust
/// 解码子进程输出:先严格 UTF-8,失败回退 GBK(Windows 中文控制台)。
fn decode_console(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => encoding_rs::GBK.decode(bytes).0.into_owned(),
    }
}
```

## jdbc.rs 诊断日志埋点

### `call()`(spawn 主路径)

spawn 前:
```rust
log::info!("spawn connector: java -jar {} (drivers_dir={:?})", self.connector_path, self.drivers_dir);
log::info!("connector request: {}", redact_password(&request));  // password 脱敏
```

spawn 结果:成功记 pid(如可得);失败记完整 io error(现有 map_err 处补 log::error!)。

wait_with_output 后:
```rust
log::info!("connector exit: {:?}", output.status.code());
// stdout/stderr 双解码,便于对比 UTF-8 lossy 与 GBK 真实内容
log::info!("connector stdout (utf8-lossy): {}", String::from_utf8_lossy(&output.stdout));
log::info!("connector stdout (gbk): {}", encoding_rs::GBK.decode(&output.stdout).0);
log::info!("connector stderr (utf8-lossy): {}", String::from_utf8_lossy(&output.stderr));
log::info!("connector stderr (gbk): {}", encoding_rs::GBK.decode(&output.stderr).0);
```

### 错误回传修复

现状(jdbc.rs:108-116):exit≠0 → 读 stderr(硬 UTF-8);exit=0 才 parse stdout JSON。
问题:Java 错误经 `writeError` 写到 **stdout** JSON,exit≠0 分支读不到。

改为:
1. 先尝试 parse stdout 为 JSON,若含 `{error:...}` → 回传该 error(真实业务错误,如密码错误/超时)。
2. 否则(stdout 非 JSON,通常是 JVM launcher 直接报错):用 `decode_console` 解码 stdout+stderr 回传(GBK 可读)。
3. exit≠0 且无有效信息:回传 decode_console(stderr) + exit code。

保证:JVM launcher 的中文错误("无法访问 jarfile")按 GBK 正确解码回前端,不再乱码。

### `check_java_once()`

同样用 `decode_console` 替代 `from_utf8_lossy`(第 148 行 combined),并加 spawn 失败/version 解析结果的 log。

## 影响面

- 改动文件:`crates/aqua-core/Cargo.toml`、`crates/aqua-core/src/driver/jdbc.rs`、`src-tauri/Cargo.toml`、`src-tauri/src/lib.rs`。
- 不改 connector(Java)、不改 import.rs 错误链(最里层可读了,外层三重"连接失败"前缀是既有行为,非本 task 目标)。
- 不动 `parse_java_major_version` 逻辑,测试不回归。

## 验证

- `cargo build -p aqua-core`、`cargo build`(src-tauri)、`cargo test -p aqua-core`。
- 无法在 mac 复现 Windows 乱码,但可单测 `decode_console`:UTF-8 输入原样返回、GBK 字节(如"错误"的 GBK 编码 `\xB4\xED\xCE\xF3")解出中文。
- Windows 侧真机验证由用户发版后执行,回收 `aqua.log` 定位根因。
