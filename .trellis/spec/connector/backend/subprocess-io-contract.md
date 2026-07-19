# Connector - 子进程 IO 契约(Rust ↔ Java)

## 概述

Rust 侧 `crates/aqua-core/src/driver/jdbc.rs` 通过 spawn `java -jar connector.jar` 与 Java 通信:
一次性进程,stdin 写 JSON 请求 → stdout 读 JSON 响应 → 进程退出。本文档定义这条边界上的
**编码契约、路径契约、错误协议**。这些是一整类跨语言/跨平台踩坑,新增 action 或调试连接问题前必读。

---

## 1. 编码契约:全链路 UTF-8

### 契约
- **Java 侧**: `Main.java` 开头强制 `System.setOut/setErr` 为 UTF-8,再写任何输出。
- **Rust 侧**: `serde_json::from_slice(&stdout)` 按 UTF-8 解析响应。

### Wrong vs Correct

#### Wrong(Windows 中文系统炸)
```java
public static void main(String[] args) {
    // System.out 默认跟随系统代码页,Windows 中文 = GBK
    System.out.println(MAPPER.writeValueAsString(response)); // 含中文时输出 GBK 字节
}
```
Rust 端 `serde_json::from_slice` 按 UTF-8 解析 GBK 字节 → `invalid unicode code point`。
`testConnection` 响应是 ASCII 不暴露,`getColumns` 带中文列注释时必炸。

#### Correct
```java
public static void main(String[] args) {
    System.setOut(new PrintStream(new FileOutputStream(FileDescriptor.out), true, StandardCharsets.UTF_8));
    System.setErr(new PrintStream(new FileOutputStream(FileDescriptor.err), true, StandardCharsets.UTF_8));
    // ... 之后所有输出都是 UTF-8
}
```

> **Warning**: 不要用 `-Dfile.encoding=UTF-8` 启动参数替代。JDK 17 上 `file.encoding` 不控制
> `System.out`(stdout 编码是独立属性,JDK 18+ 才默认 UTF-8)。`setOut` 直接钉死 PrintStream 编码,与 JDK 版本解耦。

### Rust 侧诊断解码
读子进程输出用 `decode_console`(先严格 UTF-8,失败回退 GBK),仅用于**错误诊断日志**——
让 JVM launcher 的本地化中文报错(GBK)在日志里可读。正常响应恒为 UTF-8,不依赖回退。

---

## 2. 路径契约:strip Windows verbatim 前缀

### 契约
传给 `java -jar <path>` 的路径、以及 `driversDir`,必须先 `strip_verbatim` 去掉 `\\?\` 前缀。

### Common Mistake: Java 打不开 `\\?\` 路径
- **Symptom**: 连接报"尝试打开文件 \\?\C:\...\connector.jar 时出现意外错误"(测试连接失败)。
- **Cause**: Tauri 的 `resource_dir()`/`app_data_dir()` 在 Windows 返回带 `\\?\` verbatim 前缀的路径
  (绕过 MAX_PATH 260 限制)。Java launcher 无法打开带此前缀的 jar。手动 `java -jar "C:\...\jar"`
  用普通路径所以成功——这正是"手动成功、应用失败"的根因。
- **Fix**: spawn 前 strip:
  ```rust
  fn strip_verbatim(path: &str) -> &str {
      path.strip_prefix(r"\\?\").unwrap_or(path)
  }
  ```
- **Prevention**: 任何从 Tauri path API 拿到、要传给外部进程(尤其 Java)的路径,都先 strip。其他平台无此前缀,no-op。

---

## 3. 错误协议:错误在 stdout,不在 stderr

### 契约
- Java 业务错误经 `writeError` 写到 **stdout** 的 JSON `{error: "..."}`(不是 stderr)。
- JVM launcher 级错误(jar 打不开等)走 stderr,且是本地化文字。
- Rust 侧错误处理顺序:先 parse stdout JSON 取 `{error}`,再看 exit code,stderr 仅兜底。

### Wrong vs Correct

#### Wrong(丢失真实报错)
```rust
if !output.status.success() {
    // Java 错误在 stdout,这里读 stderr 读不到真正原因
    return Err(format!("connector 失败: {}", String::from_utf8_lossy(&output.stderr)));
}
```

#### Correct
```rust
// 1. 优先 stdout JSON 业务错误(即使 exit≠0)
if let Ok(resp) = serde_json::from_slice::<Value>(&output.stdout) {
    if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
        return Err(DriverError::QueryFailed(err.to_string()));
    }
}
// 2. exit≠0:stdout 非 JSON(JVM launcher 报错),decode_console 后回传
if !output.status.success() {
    log::error!("connector 失败 exit={} stdout={} stderr={}", ...);
    return Err(...);
}
```

---

## 4. 日志(诊断手段)

- Rust 侧日志:自实现 `FileLogger`(`src-tauri/src/lib.rs`),app setup 时 `create_dir_all(app_log_dir)` +
  `set_boxed_logger`。**不用 tauri-plugin-log**——其 plugin setup 失败会被 Tauri 吞掉,日志静默丢失。
- Windows 路径:`%LOCALAPPDATA%\com.aqua.app\logs\aqua.log`(注意是 Local 非 Roaming)。
- 级别:`AQUA_LOG` 环境变量(info/warn/error/off,默认 info)。
- **静态过滤陷阱**: `log` crate 的 `STATIC_MAX_LEVEL` 由 cargo feature 决定。二进制 crate 必须启用
  `max_level_info`/`release_max_level_info`,否则 `log::info!` 在编译期被过滤成 no-op,运行时
  `set_max_level` 无法挽回。`src-tauri/Cargo.toml` 已锁 + `lib.rs` 有测试断言 `STATIC_MAX_LEVEL >= Info`。
  (故 debug 级当前不可用,需改 feature 重编。)

---

## 相关规范

- [Dialect 扩展规范](./dialect-extension.md) - Dialect 接口契约与 comment 补查钩子
- [aqua-core 数据库指南](../aqua-core/backend/database-guidelines.md) - Rust 侧驱动
