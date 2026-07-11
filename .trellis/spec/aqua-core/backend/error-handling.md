# Error Handling

> How errors are handled in this project.

---

## Overview

aqua-core 使用 Rust 标准 `Result<T, E>` + `thiserror` 定义错误类型。校验错误收集所有问题(不短路),对齐 legacy zod 行为;其他错误遵循 Rust 惯例用 `?` 传播。

---

## Error Types

### 已定义(schema 模块)

```rust
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// ValidationError - 业务校验错误,带 path 定位字段(对齐 legacy errors 结构)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,      // 如 "tables[0].fields[1].enum"
    pub message: String,
}

/// ParseError - 统一解析错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("JSON 反序列化失败: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("Project 校验失败: {count} 个错误", count = .0.len())]
    Validate(Vec<ValidationError>),
}
```

### 设计原则

- **thiserror 派生**: 所有错误类型加 `#[derive(Error)]`,自动实现 `std::error::Error`
- **校验错误带 path**: ValidationError 含 path + message,前端可定位具体字段,与 legacy 保持一致
- **校验收集不短路**: `validate_project` 遍历全部规则,收集所有错误到 `Vec<ValidationError>`,一次返回(对齐 zod `superRefine`)
- **serde 错误短路**: 结构层校验(缺失必填字段/类型不匹配)由 serde 处理,遇第一个错误即失败并返回 `serde_json::Error`(Rust 惯例,与 zod 不同)

---

## Error Handling Patterns

### 校验错误收集(不短路)

```rust
pub fn validate_project(project: &Project) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // 遍历所有规则,收集错误
    for (idx, item) in project.items.iter().enumerate() {
        if item.violates_rule_1() {
            errors.push(ValidationError::new(
                format!("items[{}].field", idx),
                "违反规则 1",
            ));
        }
        // 继续检查其他规则...
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### 结构错误传播(短路)

```rust
pub fn parse_project(value: serde_json::Value) -> Result<Project, ParseError> {
    let project: Project = serde_json::from_value(value)?;  // serde 错误自动转 ParseError::Deserialize
    validate_project(&project).map_err(ParseError::Validate)?;  // 校验错误转 ParseError::Validate
    Ok(project)
}
```

### 调用方处理

```rust
match parse_project(json_value) {
    Ok(project) => { /* 使用 project */ },
    Err(ParseError::Deserialize(e)) => {
        // serde 反序列化失败(结构/类型错误),单个错误
        eprintln!("JSON 格式错误: {}", e);
    },
    Err(ParseError::Validate(errors)) => {
        // 业务校验失败,多个错误
        for err in errors {
            eprintln!("[{}] {}", err.path, err.message);
        }
    },
}
```

---

## API Error Responses

当 aqua-core 被 Tauri 命令调用时,错误通过 `Result<T, String>` 返回前端(Tauri 自动 JSON 序列化):

```rust
#[tauri::command]
fn parse_schema(json: String) -> Result<Project, String> {
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;
    
    parse_project(value).map_err(|e| match e {
        ParseError::Deserialize(err) => format!("结构错误: {}", err),
        ParseError::Validate(errors) => {
            // 序列化为 JSON 数组返回前端
            serde_json::to_string(&errors).unwrap_or_else(|_| "校验失败".to_string())
        }
    })
}
```

前端解析 `ParseError::Validate` 的 JSON 数组,逐个显示错误并定位字段。

---

## Common Mistakes

### ❌ 在 thiserror `#[error]` 中直接用 `{0}` 格式化 Vec

```rust
#[error("失败: {0}")]  // Vec<T> 不实现 Display,编译错误
Validate(Vec<ValidationError>),
```

**✅ 正确**: 用命名参数 + 表达式

```rust
#[error("失败: {count} 个错误", count = .0.len())]
Validate(Vec<ValidationError>),
```

### ❌ 校验函数短路返回第一个错误

```rust
for item in items {
    if item.invalid() {
        return Err(vec![error]);  // 只返回第一个,不符合 legacy 行为
    }
}
```

**✅ 正确**: 收集全部错误

```rust
let mut errors = Vec::new();
for item in items {
    if item.invalid() {
        errors.push(error);  // 继续收集
    }
}
if !errors.is_empty() { Err(errors) } else { Ok(()) }
```

### ❌ 混淆结构校验与业务校验

结构校验(必填字段/类型匹配)交给 serde,业务校验(enum 只支持 VARCHAR / hasCode 一致性)写在 validate 函数。不要用 `Option<T>` + 手写判空来做必填校验,让 serde 类型即约束。
