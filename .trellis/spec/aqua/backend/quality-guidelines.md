# Quality Guidelines

> Code quality standards for Tauri shell layer.

---

## Overview

`src-tauri` 是薄胶水层,连接前端与 `aqua-core`。代码量小,复杂度低,遵循 Rust 标准 + Tauri 最佳实践即可。

---

## Forbidden Patterns

### ❌ 在 Tauri commands 里写业务逻辑

```rust
#[tauri::command]
fn validate_project(json: String) -> Result<Project, String> {
    let project: Project = serde_json::from_str(&json)?;
    
    // ❌ 校验逻辑写在 command 里
    for table in &project.tables {
        if table.fields.is_empty() {
            return Err("table 不能没有字段".into());
        }
    }
    
    Ok(project)
}
```

**✅ 正确**: 调 `aqua_core::schema::parse_project`(含校验)。

```rust
#[tauri::command]
fn validate_project(json: String) -> Result<Project, String> {
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    aqua_core::schema::parse_project(value)
        .map_err(|e| e.to_string())
}
```

### ❌ 同步 I/O 阻塞 Tauri 主线程

```rust
#[tauri::command]
fn load_project(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)  // ❌ 同步阻塞
        .map_err(|e| e.to_string())
}
```

**✅ 正确**: 用 `async` + `tokio::fs`。

```rust
#[tauri::command]
async fn load_project(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await  // ✅ 异步非阻塞
        .map_err(|e| e.to_string())
}
```

### ❌ unwrap/expect 在 command 里

```rust
#[tauri::command]
fn some_command() -> String {
    let data = fetch_data().unwrap();  // ❌ panic 会崩溃整个 GUI
    data
}
```

**✅ 正确**: 返回 `Result<T, String>`,传播错误到前端。

```rust
#[tauri::command]
fn some_command() -> Result<String, String> {
    let data = fetch_data().map_err(|e| e.to_string())?;
    Ok(data)
}
```

---

## Required Patterns

### ✅ Tauri commands 必须返回 Result

```rust
#[tauri::command]
async fn project_open(path: String) -> Result<Project, String> {
    // ...
}
```

即使不会失败,也返回 `Result<T, String>`,前端统一 `.then().catch()` 处理。

### ✅ 错误消息用户友好

```rust
// ❌ 裸错误
Err(e.to_string())  // "No such file or directory (os error 2)"

// ✅ 用户可理解
Err(format!("无法打开文件 {}: {}", path, e))
```

### ✅ async command 用 tokio runtime

Tauri 自动提供 tokio runtime,直接用 `async fn` + `.await`:

```rust
#[tauri::command]
async fn spawn_connector(config: DbConfig) -> Result<String, String> {
    let output = tokio::process::Command::new("java")
        .args(&["-jar", "connector.jar"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn connector 失败: {}", e))?
        .wait_with_output()
        .await
        .map_err(|e| format!("connector 执行失败: {}", e))?;
    
    String::from_utf8(output.stdout)
        .map_err(|e| format!("connector 输出非 UTF-8: {}", e))
}
```

---

## Testing Requirements

### 单元测试(可选,薄层不强求)

Tauri commands 是胶水代码,测试性价比低。重点测 `aqua-core`,不测 commands。

**例外**: CLI 参数解析(`cli.rs`)值得测试:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_generate_args() {
        let args = vec![
            "generate".to_string(),
            "ddl".to_string(),
            "--schema=test.json".to_string(),
        ];
        let config = parse_generate_args(&args[1..]).unwrap();
        assert_eq!(config.output_type, "ddl");
        assert_eq!(config.schema_path, "test.json");
    }
}
```

### 集成测试(手动)

- **GUI**: 手动启动 `cargo tauri dev`,点前端按钮验证 commands
- **CLI**: `cargo run -- generate ddl --schema=test.json`,验证 stdout 输出

---

## Code Review Checklist

### Tauri commands

- [ ] 返回 `Result<T, String>`,不 panic
- [ ] I/O 用 async(tokio::fs / tokio::process)
- [ ] 业务逻辑委托给 `aqua-core`,不在 command 里实现
- [ ] 错误消息用户友好(含文件名/表名等上下文)
- [ ] 复杂错误(如 `Vec<ValidationError>`)序列化为 JSON 字符串返回

### CLI

- [ ] 无 GUI 依赖(不启动 Tauri Builder)
- [ ] stdout 输出结果,stderr 输出错误
- [ ] 用 `anyhow::Result` 简化错误处理

### 质量门禁

- [ ] `cargo clippy -p aqua -- -D warnings` 无 warning
- [ ] `cargo build --release` 通过
- [ ] 手动测试 GUI + CLI 主流程

---

## Tauri-Specific Patterns

### State 管理(按需)

若需共享状态(如当前打开的 project),用 Tauri State:

```rust
use tauri::State;
use std::sync::Mutex;

struct AppState {
    current_project: Mutex<Option<Project>>,
}

#[tauri::command]
fn get_current_project(state: State<AppState>) -> Option<Project> {
    state.current_project.lock().unwrap().clone()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            current_project: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![get_current_project])
        .run(tauri::generate_context!())
        .unwrap();
}
```

**当前**: aqua 暂无全局状态需求,前端自管项目 JSON。

### 文件对话框

```rust
use tauri::api::dialog;

#[tauri::command]
async fn pick_schema_file() -> Option<String> {
    dialog::FileDialogBuilder::new()
        .add_filter("JSON", &["json"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}
```

---

## Examples

**当前状态**: `src-tauri/src/lib.rs` - 占位 `greet` command。

**参考模式**: 
- GUI/CLI 双模式: `lib.rs::run()`
- 错误映射: `aqua_core::schema::ParseError` → `String`
- async I/O: `tokio::fs` / `tokio::process`
