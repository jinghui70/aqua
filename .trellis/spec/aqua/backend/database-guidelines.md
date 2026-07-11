# Database Guidelines

> Database-related patterns in Tauri shell layer.

---

## Overview

`src-tauri` **不直接操作数据库**。所有数据库交互委托给 `aqua-core`,壳层只负责:
1. **spawn connector**: 启动 Java connector 子进程,传递 DB 配置
2. **文件 I/O**: 读写 schema.json / .aqua 文件(通过 `tokio::fs`)

---

## No Database Access

Tauri 壳层无 ORM / 无 SQL 查询 / 无连接池。

```rust
// ❌ 错误:在 src-tauri 里直接连数据库
use mysql_async::Pool;

#[tauri::command]
async fn list_tables() -> Result<Vec<String>, String> {
    let pool = Pool::new("mysql://...");  // ❌ 数据库逻辑属于 aqua-core
    // ...
}
```

**✅ 正确**: 调用 `aqua-core` Driver trait。

```rust
use aqua_core::driver::{create_driver, DbConfig};

#[tauri::command]
async fn list_tables(config: DbConfig) -> Result<Vec<String>, String> {
    let driver = create_driver(config)
        .map_err(|e| format!("创建驱动失败: {}", e))?;
    
    driver.list_tables("public").await
        .map_err(|e| e.to_string())
}
```

---

## Spawn Connector Pattern

唯一与数据库相关的操作是 **spawn Java connector**:

```rust
use tokio::process::Command;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tauri::command]
async fn test_oracle_connection(config: DbConfig) -> Result<String, String> {
    // 构造 stdin JSON
    let request = serde_json::json!({
        "action": "testConnection",
        "host": config.host,
        "port": config.port,
        "user": config.user,
        "password": config.password,
        "database": config.database,
    });
    
    // spawn connector.jar
    let mut child = Command::new("java")
        .args(&["-jar", "connector.jar"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 connector 失败(需 JDK 17+): {}", e))?;
    
    // 写 stdin
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(request.to_string().as_bytes()).await
        .map_err(|e| format!("写 stdin 失败: {}", e))?;
    drop(stdin);
    
    // 读 stdout
    let output = child.wait_with_output().await
        .map_err(|e| format!("connector 执行失败: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "连接失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    String::from_utf8(output.stdout)
        .map_err(|e| format!("connector 输出非 UTF-8: {}", e))
}
```

**职责**: 仅 spawn 子进程 + stdin/stdout 通信,不解析数据库协议。

---

## File I/O (schema.json / .aqua)

```rust
#[tauri::command]
async fn load_schema(path: String) -> Result<Project, String> {
    let json = tokio::fs::read_to_string(&path).await
        .map_err(|e| format!("读取 {} 失败: {}", path, e))?;
    
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    aqua_core::schema::parse_project(value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_schema(path: String, project: Project) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("JSON 序列化失败: {}", e))?;
    
    tokio::fs::write(&path, json).await
        .map_err(|e| format!("保存 {} 失败: {}", path, e))
}
```

**职责**: 异步文件读写,不解析 schema 业务逻辑。

---

## Summary

- **无直接数据库访问**: 调用 `aqua-core` Driver trait
- **spawn connector**: 启动 Java 子进程,stdin/stdout JSON 通信
- **文件 I/O**: `tokio::fs` 读写 schema.json / .aqua

参考 `aqua-core/backend/database-guidelines.md` 了解真实数据库交互模式。
