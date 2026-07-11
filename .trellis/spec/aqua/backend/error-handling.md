# Error Handling

> How errors are handled in Tauri shell layer.

---

## Overview

`src-tauri` 错误处理极简:接收 `aqua-core` 的 `Result`,映射为 Tauri 兼容的 `Result<T, String>`,传递给前端。

**原则**: 
- 不定义自定义错误类型(薄层无需)
- 用 `anyhow::Result` 简化 CLI 错误处理
- Tauri commands 统一返回 `Result<T, String>`

---

## Error Types

### Tauri commands: `Result<T, String>`

```rust
#[tauri::command]
async fn project_open(path: String) -> Result<Project, String> {
    // 所有错误最终 map_err 为 String
}
```

**原因**: Tauri 自动序列化 `String` 错误到前端,复杂类型需手动处理。

### CLI: `anyhow::Result<()>`

```rust
// cli.rs
use anyhow::{Context, Result};

pub fn run_generate(args: &[String]) -> Result<()> {
    let config = parse_args(args)
        .context("参数解析失败")?;
    
    let json = std::fs::read_to_string(&config.schema)
        .context(format!("读取 {} 失败", config.schema))?;
    
    let project = aqua_core::schema::parse_project(
        serde_json::from_str(&json)?
    )?;
    
    println!("{}", generate_output(&project)?);
    Ok(())
}
```

**原因**: `anyhow` 提供 `.context()` 附加上下文,stderr 输出完整错误链。

---

## Error Handling Patterns

### Tauri command 错误映射

```rust
use aqua_core::schema::{parse_project, ParseError};

#[tauri::command]
fn validate(json: String) -> Result<Project, String> {
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    parse_project(value).map_err(|e| match e {
        ParseError::Deserialize(err) => {
            format!("schema 结构错误: {}", err)
        },
        ParseError::Validate(errors) => {
            // 复杂错误序列化为 JSON 字符串
            serde_json::to_string(&errors)
                .unwrap_or_else(|_| format!("校验失败: {} 个错误", errors.len()))
        },
    })
}
```

**模式**: 
1. I/O 错误 → `format!("操作失败: {}", e)`
2. aqua-core 错误 → match 分支格式化
3. 复杂错误 → JSON 序列化为字符串

### CLI 错误传播

```rust
fn main() {
    if let Err(e) = aqua::cli::run(std::env::args().collect()) {
        eprintln!("错误: {:?}", e);  // anyhow 自动打印错误链
        std::process::exit(1);
    }
}
```

**模式**: 
- 用 `?` 传播,不手动 match
- main 里统一 `eprintln!` + `exit(1)`
- `.context()` 附加操作上下文

---

## API Error Responses

Tauri commands 返回 `Result<T, String>`,前端收到:

**成功**:
```json
{"data": {...}}
```

**失败**:
```json
{"error": "无法打开文件 /path/to/file: No such file"}
```

前端解析:

```typescript
import { invoke } from "@tauri-apps/api/core";

try {
  const project = await invoke<Project>("project_open", { path });
  // 成功
} catch (err) {
  // err 是 String,可能是 JSON(复杂错误)或纯文本
  try {
    const errors = JSON.parse(err as string);  // 尝试解析 ValidationError[]
    showValidationErrors(errors);
  } catch {
    showSimpleError(err as string);  // 简单错误
  }
}
```

---

## Common Mistakes

### ❌ unwrap/expect 在 command 里

```rust
#[tauri::command]
fn load(path: String) -> Project {
    let json = std::fs::read_to_string(&path).unwrap();  // ❌ 文件不存在会崩溃整个 GUI
    serde_json::from_str(&json).unwrap()
}
```

**✅ 正确**: 返回 `Result`,传播错误到前端。

```rust
#[tauri::command]
fn load(path: String) -> Result<Project, String> {
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))
}
```

### ❌ 直接返回 aqua-core 错误类型

```rust
use aqua_core::schema::ParseError;

#[tauri::command]
fn validate(json: String) -> Result<Project, ParseError> {  // ❌ ParseError 无法序列化
    parse_project(serde_json::from_str(&json)?)
}
```

**✅ 正确**: 映射为 `String`。

```rust
#[tauri::command]
fn validate(json: String) -> Result<Project, String> {
    let value = serde_json::from_str(&json)
        .map_err(|e| e.to_string())?;
    parse_project(value).map_err(|e| e.to_string())
}
```

### ❌ 吞没错误细节

```rust
.map_err(|_| "操作失败".to_string())  // ❌ 丢失原始错误
```

**✅ 正确**: 保留错误信息。

```rust
.map_err(|e| format!("操作失败: {}", e))
```

---

## Logging vs Returning

**原则**: 
- **Tauri commands**: 不记日志,直接返回 `Err(String)` 给前端(前端决定如何展示)
- **CLI**: 严重错误记 error 日志,然后 `return Err`

```rust
// Tauri command: 不记日志
#[tauri::command]
fn load(path: String) -> Result<Project, String> {
    // 不调 log::error!,直接返回错误
    std::fs::read_to_string(&path)
        .map_err(|e| format!("读取失败: {}", e))?;
    // ...
}

// CLI: 记日志
pub fn run_generate(args: &[String]) -> anyhow::Result<()> {
    let project = load_schema(&args[0])
        .context("加载 schema 失败")?;
    
    if let Err(e) = generate(&project) {
        error!("生成失败: {}", e);  // CLI 记日志
        return Err(e);
    }
    Ok(())
}
```

---

## Examples

### 文件 I/O 错误

```rust
#[tauri::command]
async fn save_project(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content).await
        .map_err(|e| format!("保存文件 {} 失败: {}", path, e))
}
```

### spawn connector 错误

```rust
#[tauri::command]
async fn test_connection(config: DbConfig) -> Result<String, String> {
    let output = tokio::process::Command::new("java")
        .args(&["-jar", "connector.jar"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 connector 失败(需 JDK 17+): {}", e))?
        .wait_with_output()
        .await
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

### CLI 错误链

```rust
// main 里统一处理
fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {:?}", e);  // anyhow 打印完整错误链
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "generate" {
        aqua::cli::run_generate(&args[2..])
            .context("generate 命令执行失败")?;
    } else {
        aqua::run_gui();  // GUI 模式
    }
    
    Ok(())
}
```
