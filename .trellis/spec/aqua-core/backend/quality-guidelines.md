# Quality Guidelines

> Code quality standards for backend development.

---

## Overview

aqua-core 遵循 Rust 社区惯例 + 项目特定约定。所有代码必须通过 `cargo clippy -D warnings` / `cargo fmt --check` / `cargo test`,无例外。

---

## Forbidden Patterns

### ❌ 跳过 serde derive,手写 Serialize/Deserialize

**为什么**: serde derive 已覆盖 99% 场景,手写易出错且难维护。

**例外**: 极特殊序列化逻辑(目前无此场景)。

### ❌ 用 `unwrap()` / `expect()` 处理可恢复错误

```rust
let project: Project = serde_json::from_value(value).unwrap();  // ❌ 会 panic
```

**正确**: 用 `?` 传播或 `match` 处理

```rust
let project: Project = serde_json::from_value(value)?;  // ✅
```

**例外**: 测试代码可用 `expect("测试数据应合法")`,生产代码禁止。

### ❌ 在类型定义文件放业务逻辑

类型定义(struct/enum + serde derive)与业务逻辑(校验/转换)分离。

**正确**: 类型在 `field.rs`,校验在 `validate.rs`。

### ❌ 忽略 clippy 建议

所有 clippy warning 必须修复,不许 `#[allow(clippy::xxx)]` 压制(除非有文档说明的充分理由)。

---

## Required Patterns

### ✅ 所有公共类型加 serde derive

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub prop: String,
    #[serde(rename = "dataType")]  // JSON 驼峰 ↔ Rust 蛇形
    pub data_type: DataType,
}
```

**必须**: Debug(调试) / Clone(灵活传递) / PartialEq(测试断言) / Serialize + Deserialize(JSON 序列化)。

### ✅ JSON 字段名用 `#[serde(rename)]` 对齐 legacy

Rust 字段蛇形命名,JSON 保持驼峰(对齐 legacy TS):

```rust
#[serde(rename = "basePackage")]
pub base_package: String,
```

### ✅ 可选字段加 `#[serde(skip_serializing_if = "Option::is_none")]`

JSON 中省略 `null` 字段,保持简洁:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub comment: Option<String>,
```

### ✅ 枚举变体用 `#[serde(rename_all)]` 统一大小写

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]       // VARCHAR → "VARCHAR"
pub enum DataType { Varchar, Clob }

#[serde(rename_all = "SCREAMING_SNAKE_CASE")]  // InsertUpdate → "INSERT_UPDATE"
pub enum GenerateTiming { Insert, InsertUpdate }
```

### ✅ 错误类型用 thiserror

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("反序列化失败: {0}")]
    Deserialize(#[from] serde_json::Error),
}
```

---

## Testing Requirements

### 必需测试

1. **正向用例**: 完整有效输入,验证成功路径(如 `test_accepts_complete_valid_project`)
2. **负向用例**: 每个校验规则至少一个失败场景(如 `test_rejects_enum_on_non_varchar_datatype`)
3. **serde 往返**: `JSON → Rust → JSON` 等价性(验证序列化无损)

### 测试组织

- **集成测试**: `tests/<module>.rs`,测试公共 API
- **fixtures**: `tests/fixtures/*.json`,从 legacy 移植或手写,命名清晰(valid-xxx / invalid-xxx)
- **辅助函数**: `load_fixture` 等,放测试文件内,不导出

### 断言风格

```rust
// ✅ 语义清晰的断言消息
assert!(result.is_ok(), "valid-full 应通过校验");
assert!(errors.iter().any(|e| e.message.contains("enum 只支持 VARCHAR")),
        "应包含 'enum 只支持 VARCHAR' 错误");

// ❌ 无消息的裸断言
assert!(result.is_ok());  // 失败时无上下文
```

---

## Code Review Checklist

### 类型定义

- [ ] 所有 pub struct/enum 加 `Debug, Clone, PartialEq, Serialize, Deserialize`
- [ ] JSON 字段名用 `#[serde(rename)]` 对齐 legacy
- [ ] 可选字段加 `skip_serializing_if = "Option::is_none"`
- [ ] 枚举变体用 `#[serde(rename_all)]` 统一大小写
- [ ] 文件名避开 Rust 关键字(enum → enum_def)

### 校验逻辑

- [ ] 结构校验(必填/类型)交给 serde,业务校验(语义规则)在 validate 函数
- [ ] validate 函数收集全部错误(不短路),返回 `Vec<ValidationError>`
- [ ] ValidationError 带 path(数组索引 + 字段名),如 `"tables[0].fields[1].enum"`

### 错误处理

- [ ] 错误类型用 thiserror `#[derive(Error)]`
- [ ] 生产代码不用 `unwrap()` / `expect()`,用 `?` 或 `match`
- [ ] `#[error]` 格式化 Vec 时用命名参数(避免 Display trait 错误)

### 测试

- [ ] 正向用例覆盖主流程
- [ ] 每个校验规则有负向用例
- [ ] serde 往返测试(JSON → Rust → JSON 等价)
- [ ] 断言带清晰消息

### 质量门禁

- [ ] `cargo test -p aqua-core` 全绿
- [ ] `cargo clippy -p aqua-core -- -D warnings` 无 warning
- [ ] `cargo fmt -p aqua-core -- --check` 通过
- [ ] 新增公共 API 有文档注释(`///`)

---

## Examples

**参考范式**: `crates/aqua-core/src/schema/` - 完整的类型定义、校验分离、错误处理、测试覆盖,后续模块照此模式。
