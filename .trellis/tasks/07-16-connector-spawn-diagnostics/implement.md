# Implement 计划

## 步骤

1. **依赖**
   - `crates/aqua-core/Cargo.toml`:加 `log = "0.4"`、`encoding_rs = "0.8"`。
   - `src-tauri`:`cargo add tauri-plugin-log`(解析 v2 具体版本)。

2. **src-tauri 日志 sink**
   - `lib.rs::run()` 在 `tauri::Builder::default()` 后、`.plugin(tauri_plugin_dialog::init())` 附近注册 `tauri_plugin_log`,LogDir target,file_name "aqua",level Info。

3. **jdbc.rs**
   - 加 `decode_console` 私有 fn + `redact_password(&Value) -> String` 私有 fn。
   - `call()`:spawn 前后加 log 埋点(argv/drivers_dir/脱敏 request/exit/stdout+stderr 双解码)。
   - 重写 exit 后错误处理:优先 parse stdout `{error}`,否则 decode_console 回传。
   - `check_java_once()`:`from_utf8_lossy` → `decode_console`,加 spawn/version log。

4. **测试**
   - jdbc.rs `#[cfg(test)]` 加 `test_decode_console`:UTF-8 直通 + GBK 字节解码。

5. **验证**
   - `cargo test -p aqua-core`(含新测 + 旧 parse_java_major_version)。
   - `cargo build`(workspace,确认 src-tauri 编译过、plugin 注册无误)。

## 交付说明(给用户)

- Windows 日志路径:`%LOCALAPPDATA%\com.aqua.app\logs\aqua.log`。
- 发版后复现一次"测试连接",把该文件发回,依据 `connector exit` / `stdout(gbk)` / `stderr(gbk)` / argv 定位根因。
- 黑窗口本轮保留(证据),定位后另开 task 处理。
