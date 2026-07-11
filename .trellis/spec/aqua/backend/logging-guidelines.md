# Logging Guidelines

> How logging is done in Tauri shell layer.

---

## Overview

`src-tauri` 作为薄层,日志极少:
- **Tauri commands**: 不记日志,错误直接返回前端
- **CLI**: 用 `log` crate 记录执行过程
- **启动/关闭**: info 级别记录应用生命周期

**日志库**: `log` crate + Tauri 内置日志后端(自动写入用户目录)。

---

## Log Levels

### info (默认)

**用途**: 应用启动/关闭/CLI 命令执行。

```rust
// lib.rs
pub fn run() {
    info!("aqua v{} starting", env!("CARGO_PKG_VERSION"));
    
    if is_cli_mode() {
        info!("CLI mode: {:?}", args);
        cli::run_generate(&args).expect("CLI 执行失败");
        info!("CLI execution complete");
    } else {
        tauri::Builder::default()
            .setup(|_| {
                info!("GUI mode initialized");
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("Tauri 启动失败");
    }
}
```

### debug (开发期)

**用途**: Tauri 内部事件/IPC 调用追踪(由 Tauri 自动记录,不手动写)。

### error (关键错误)

**用途**: 仅 CLI 执行失败时记录(Tauri commands 错误不记,直接返回前端)。

```rust
// cli.rs
pub fn run_generate(args: &[String]) -> anyhow::Result<()> {
    let config = parse_args(args)?;
    
    match generate(&config) {
        Ok(output) => {
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            error!("generate 失败: {:?}", e);  // CLI 才记 error
            Err(e)
        }
    }
}
```

---

## What to Log

### ✅ 应该记录

- **应用启动**: `info!("aqua v{} starting", version)`
- **CLI 命令执行**: `info!("executing generate: dialect={}", dialect)`
- **CLI 错误**: `error!("command failed: {}", err)`

### ❌ 不应该记录

- **Tauri commands 执行**: 不记(由前端决定如何处理错误)
- **用户数据**: 不记(schema 内容/数据库连接密码)
- **高频事件**: 不记(每次 invoke 调用)

---

## What NOT to Log

### 敏感信息

```rust
// ❌ 泄露密码
debug!("spawning connector with config: {:?}", db_config);  // 含 password

// ✅ 脱敏
debug!("spawning connector: {}@{}:{}", db_config.user, db_config.host, db_config.port);
```

### Tauri command 参数/返回值

```rust
// ❌ 记录用户数据
#[tauri::command]
fn save_project(content: String) -> Result<(), String> {
    debug!("saving: {}", content);  // 可能含敏感 schema
    // ...
}

// ✅ 不记录,或仅记操作
#[tauri::command]
fn save_project(path: String, content: String) -> Result<(), String> {
    // 不记日志,直接执行
    tokio::fs::write(&path, content).await
        .map_err(|e| format!("保存失败: {}", e))
}
```

---

## Log Configuration

### 开发期(cargo tauri dev)

Tauri 自动输出到终端:

```bash
cargo tauri dev
# [INFO aqua] aqua v0.1.0 starting
# [INFO aqua] GUI mode initialized
```

### 生产(打包后)

日志自动写入:
- macOS: `~/Library/Logs/aqua/aqua.log`
- Windows: `%APPDATA%\aqua\logs\aqua.log`
- Linux: `~/.local/share/aqua/logs/aqua.log`

**配置**(可选,在 `lib.rs` setup 里):

```rust
tauri::Builder::default()
    .setup(|app| {
        // Tauri 默认配置已足够,无需手动设置
        Ok(())
    })
    .run(tauri::generate_context!())
```

---

## Error Logging Pattern

### Tauri commands: 不记日志

```rust
#[tauri::command]
fn load(path: String) -> Result<Project, String> {
    // 错误直接返回,不记日志(前端决定如何展示)
    let json = tokio::fs::read_to_string(&path).await
        .map_err(|e| format!("读取失败: {}", e))?;
    
    parse_json(&json)
        .map_err(|e| e.to_string())
}
```

**原因**: 前端收到错误后可能弹窗/提示,记日志会重复。

### CLI: 记录失败

```rust
pub fn run_generate(args: &[String]) -> anyhow::Result<()> {
    info!("starting generate: {:?}", args);
    
    let result = generate_internal(args);
    
    match result {
        Ok(output) => {
            println!("{}", output);
            info!("generate complete");
            Ok(())
        }
        Err(e) => {
            error!("generate failed: {:?}", e);  // 记录到日志文件
            Err(e)  // 同时返回给 main(打印 stderr)
        }
    }
}
```

---

## Common Mistakes

### ❌ 在 Tauri command 里记 error

```rust
#[tauri::command]
fn load(path: String) -> Result<Project, String> {
    match std::fs::read_to_string(&path) {
        Err(e) => {
            error!("load failed: {}", e);  // ❌ 前端也会显示错误,重复
            Err(e.to_string())
        }
        Ok(content) => parse(content),
    }
}
```

**✅ 正确**: 不记日志,直接返回错误。

```rust
#[tauri::command]
fn load(path: String) -> Result<Project, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取失败: {}", e))?;
    parse(content)
}
```

### ❌ 记录用户数据

```rust
info!("loaded project: {:?}", project);  // ❌ project 可能含敏感表名/字段名
```

**✅ 正确**: 只记统计。

```rust
info!("loaded project: {} tables", project.tables.len());
```

---

## Examples

### 应用启动日志

```rust
// lib.rs
pub fn run() {
    info!("aqua v{} starting on {:?}", 
          env!("CARGO_PKG_VERSION"), 
          std::env::consts::OS);
    
    // ...
}
```

### CLI 执行日志

```rust
// cli.rs
pub fn run_generate(args: &[String]) -> anyhow::Result<()> {
    let config = parse_args(args)?;
    info!("generate {} from {}", config.dialect, config.schema);
    
    let output = generate(&config)?;
    println!("{}", output);
    
    info!("generated {} lines", output.lines().count());
    Ok(())
}
```

### spawn connector(不记敏感信息)

```rust
#[tauri::command]
async fn test_connection(config: DbConfig) -> Result<String, String> {
    // 不记 config.password
    debug!("testing connection: {}@{}:{}", config.user, config.host, config.port);
    
    let output = spawn_connector(&config).await?;
    
    // 不记录详细输出(可能含表名)
    debug!("connection test complete");
    
    Ok(output)
}
```

---

## Summary

`src-tauri` 日志极简:
- **启动/CLI**: 记 info/error
- **Tauri commands**: 不记(错误返回前端)
- **敏感数据**: 永不记录
