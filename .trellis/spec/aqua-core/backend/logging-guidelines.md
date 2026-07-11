# Logging Guidelines

> How logging is done in aqua-core.

---

## Overview

aqua-core 是**纯逻辑库**,日志仅用于调试与错误追踪,不做结构化日志/日志聚合。上层 `src-tauri` 可选择性转发日志到 Tauri 控制台或文件。

**日志库**: Rust 标准 `log` crate(facade) + `env_logger`(简单后端,开发期用)。生产环境由 Tauri 配置日志后端。

---

## Log Levels

```rust
use log::{trace, debug, info, warn, error};
```

### trace (最详细,默认关闭)

**用途**: 函数调用链路/变量值追踪,仅开发期调试复杂逻辑时开启。

```rust
trace!("validate_project: checking table[{}] field[{}]", table_idx, field_idx);
```

**何时用**: 排查深层 bug,需要完整执行路径时。**生产禁用**(性能开销大)。

### debug (开发期默认)

**用途**: 子模块入口/出口、关键分支判断、非敏感中间结果。

```rust
debug!("parse_project: loaded {} tables, {} enums", tables.len(), enums.len());
debug!("driver factory: creating MySQL driver for {}@{}", user, host);
```

**何时用**: 开发期追踪程序流程,生产环境可按需开启(不含敏感数据)。

### info (生产默认)

**用途**: 重要操作成功/失败的高层事件,用户可见的关键节点。

```rust
info!("schema imported from {}: {} tables", db_url, table_count);
info!("DDL generated: {} files, {} lines", file_count, line_count);
```

**何时用**: 
- 用户操作完成(导入/生成/diff)
- 连接建立/断开
- 文件读写成功

**不用于**: 循环内/高频调用(会刷屏)。

### warn (需关注但非错误)

**用途**: 降级/回退/兼容性问题、用户配置不推荐但可运行。

```rust
warn!("table {} has no primary key, generators may produce non-optimal code", table_code);
warn!("JDK version {} detected, recommend 17+ for better performance", jdk_version);
```

**何时用**:
- 自动修正/降级处理(如缺 code 字段时自动生成)
- 用户配置问题但不影响核心功能
- 废弃 API 调用

### error (需立即处理)

**用途**: 操作失败、数据损坏、致命错误(但不 panic)。

```rust
error!("failed to connect to {}@{}: {}", user, host, err);
error!("schema validation failed: {} errors", errors.len());
```

**何时用**:
- 连接失败/超时
- 文件读写失败
- 校验失败(但不 panic,返回 `Result::Err`)

**不用于**: 预期内的业务错误(如"用户取消操作"),那些直接返回 `Result::Err` 不记日志。

---

## Structured Logging

aqua-core 当前**不强制结构化日志**(无 JSON/键值对),用简单文本 + 占位符:

```rust
// ✅ 简单清晰
info!("imported {} tables from {}", count, db_name);

// ❌ 过度结构化(aqua 非长期运行服务,无需机器解析日志)
info!(target: "import", table_count = count, source = db_name; "import complete");
```

**例外**: 若后续引入 `tracing` crate 做性能追踪,可用结构化字段,但核心逻辑保持简单 `log` 即可。

---

## What to Log

### ✅ 应该记录

- **连接事件**: 连接建立/失败/超时(含 `user@host:port`,隐藏密码)
- **文件 I/O**: schema.json 读写/dataset 创建/DDL 输出(含路径)
- **校验结果**: `validate_project` 成功/失败+错误数(不记录全部错误详情,那些返回给调用方)
- **性能关键点**: 大表导入耗时(debug 级别)
- **降级/回退**: warn 级别,说明原因+影响

### ❌ 不应该记录

- **密码/密钥**: 连接字符串脱敏,只记 `user@host:port`
- **完整 SQL**: 可能含敏感数据,只记 SQL 类型(如 "SELECT columns from INFORMATION_SCHEMA")
- **用户数据内容**: dataset 导入时不记录行内容,只记行数
- **高频循环**: 如"处理第 N 行"(刷屏无意义),改为每 1000 行记一次或仅记总数

---

## What NOT to Log

### 敏感信息(必须脱敏)

```rust
// ❌ 泄露密码
debug!("connecting to {}", connection_string);  // 含 password=xxx

// ✅ 脱敏
debug!("connecting to {}@{}:{}/{}", config.user, config.host, config.port, config.database);
```

### PII(个人身份信息)

aqua 导入的表可能含用户姓名/身份证/手机号,日志**禁止记录行内容**:

```rust
// ❌ 泄露 PII
debug!("imported row: {:?}", row);  // row 可能含敏感字段

// ✅ 只记统计
info!("imported {} rows from table {}", row_count, table_name);
```

### 噪音日志

```rust
// ❌ 循环内高频日志
for field in fields {
    debug!("processing field {}", field.code);  // 刷屏
}

// ✅ 汇总记录
debug!("processing {} fields in table {}", fields.len(), table.code);
```

---

## Error Logging Pattern

错误日志 + Result 返回双轨制:

```rust
pub fn parse_project(value: serde_json::Value) -> Result<Project, ParseError> {
    let project: Project = match serde_json::from_value(value) {
        Ok(p) => p,
        Err(e) => {
            error!("schema deserialization failed: {}", e);  // 记日志
            return Err(ParseError::Deserialize(e));          // 返回错误
        }
    };
    
    if let Err(errors) = validate_project(&project) {
        error!("schema validation failed: {} errors", errors.len());
        // 不记录全部 errors 详情(返回给调用方展示给用户)
        return Err(ParseError::Validate(errors));
    }
    
    Ok(project)
}
```

**原则**: 
- error 日志记录**事件发生**(含关键上下文,如文件名/表名/错误数)
- 详细错误信息(如全部 ValidationError)通过 `Result::Err` 返回,不刷日志
- 调用方决定如何展示错误(UI 弹窗 / CLI 打印)

---

## Log Configuration

### 开发期

```bash
# 终端设置环境变量
export RUST_LOG=aqua_core=debug,info

cargo test  # 测试输出带 debug 日志
cargo run   # 开发运行带 debug 日志
```

### 生产(Tauri 打包后)

`src-tauri` 配置日志后端,写入用户目录:

```rust
// src-tauri/src/main.rs
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    .target(env_logger::Target::Pipe(Box::new(log_file)))
    .init();
```

日志路径: 
- macOS: `~/Library/Application Support/aqua/logs/aqua.log`
- Windows: `%APPDATA%\aqua\logs\aqua.log`
- Linux: `~/.local/share/aqua/logs/aqua.log`

---

## Examples

### 连接日志(脱敏)

```rust
info!("connecting to MySQL: {}@{}:{}", user, host, port);
match driver.test_connection().await {
    Ok(_) => info!("connection established"),
    Err(e) => {
        error!("connection failed: {}", e);  // e.to_string() 不含密码
        return Err(e);
    }
}
```

### 导入日志(汇总统计)

```rust
info!("importing schema from {}", db_name);
let tables = driver.list_tables(schema).await?;
debug!("found {} tables", tables.len());

for table in tables {
    let columns = driver.get_columns(&table).await?;
    debug!("table {}: {} columns", table, columns.len());  // 每表一条,不刷屏
}

info!("import complete: {} tables imported", tables.len());
```

### 校验日志(错误计数)

```rust
match validate_project(&project) {
    Ok(_) => info!("schema validation passed"),
    Err(errors) => {
        error!("schema validation failed: {} errors", errors.len());
        // errors 详情返回给调用方,不打印到日志(避免刷屏 + 可能含敏感字段名)
        return Err(ParseError::Validate(errors));
    }
}
```

---

## Common Mistakes

### ❌ 在 Result 错误路径重复记日志

```rust
// ❌ 调用方和被调用方都记 error,重复
fn inner() -> Result<()> {
    error!("failed");  // 这里记一次
    Err(...)
}

fn outer() -> Result<()> {
    inner().map_err(|e| {
        error!("inner failed: {}", e);  // 又记一次,重复
        e
    })
}
```

**✅ 正确**: 只在最外层记 error,内层函数只返回 `Result::Err`,不记日志(或仅 debug 级别)。

### ❌ 循环内 info/warn/error

```rust
for field in fields {
    info!("field {} validated", field.code);  // ❌ 1000 个字段刷 1000 条
}
```

**✅ 正确**: 循环外汇总,或每 N 次记一条。

```rust
debug!("validating {} fields", fields.len());  // 循环前一次
// 循环内不记日志
info!("validation complete: {} fields", fields.len());  // 循环后一次
```

---

## Tools

- **日志库**: `log` crate(facade) + `env_logger`(开发) / Tauri 日志后端(生产)
- **查看日志**: 生产环境从 `~/Library/Application Support/aqua/logs/` 读取
- **过滤日志**: `RUST_LOG=aqua_core::schema=trace` 单模块 trace 级别
