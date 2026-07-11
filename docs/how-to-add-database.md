# 如何新增数据库支持

本文档描述如何为 aqua 新增一个数据库方言支持。

---

## 概述

aqua 支持两类数据库方言:

1. **内置方言**(MySQL/PostgreSQL) - native Rust 驱动,类型映射硬编码
2. **JDBC 方言**(Oracle/DM/KingBase/GBase/H2/...) - 通过 `connector.jar`,类型映射外置

新增数据库时,根据是否有成熟的 Rust 驱动选择实现方式。

---

## 方式一: JDBC 方言(推荐,快速扩展)

**适用**: 信创数据库(达梦/金仓/南大通用)、Oracle、SQL Server、DB2 等有 JDBC 驱动的数据库。

### 1. 添加 DDL 生成器支持

**文件**: `crates/aqua-core/src/generators/ddl/types.rs`

在 `map_xxx` 函数中添加新方言的类型映射:

```rust
/// 达梦数据库类型映射示例
fn map_dm(data_type: DataType, length: Option<u32>, precision: Option<u32>, scale: Option<u32>) -> String {
    match data_type {
        DataType::Varchar => format!("VARCHAR({})", length.unwrap_or(255)),
        DataType::Clob => "CLOB".to_string(),
        DataType::Tinyint => "TINYINT".to_string(),
        DataType::Int => "INT".to_string(),
        DataType::Long => "BIGINT".to_string(),
        DataType::Decimal => {
            if let Some(p) = precision {
                format!("DECIMAL({}, {})", p, scale.unwrap_or(0))
            } else {
                "DECIMAL".to_string()
            }
        }
        DataType::Date => "DATE".to_string(),
        DataType::Datetime => "TIMESTAMP".to_string(),
        DataType::Blob => "BLOB".to_string(),
    }
}
```

然后在 `map_type` 函数的 `Dialect::Jdbc` 分支注册:

```rust
Dialect::Jdbc { name } => match name.as_str() {
    "oracle" => map_oracle(data_type, length, precision, scale),
    "h2" => map_h2(data_type, length, precision, scale),
    "dm" => map_dm(data_type, length, precision, scale),  // 新增
    _ => format!("UNKNOWN_{:?}", data_type),
},
```

### 2. 配置 COMMENT 语法

**文件**: `crates/aqua-core/src/generators/ddl/table.rs`

更新 `inline_column_comment` 和 `inline_table_comment` 函数:

```rust
fn inline_column_comment(dialect: &Dialect) -> bool {
    match dialect {
        Dialect::Mysql => true,
        Dialect::Postgresql => false,
        Dialect::Jdbc { name } => matches!(name.as_str(), "h2" | "dm" | "gbase"),  // 达梦支持内联注释
    }
}
```

### 3. 添加测试

**文件**: `crates/aqua-core/tests/generators_ddl.rs`

```rust
#[test]
fn test_dm_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(&project, &DdlOptions {
        dialect: Dialect::Jdbc { name: "dm".to_string() },
        ..Default::default()
    });

    // 验证达梦特有类型
    assert!(ddl.contains("VARCHAR"));
    assert!(ddl.contains("BIGINT"));
    assert!(ddl.contains("COMMENT"));  // 达梦支持 COMMENT
}
```

### 4. 实现 JDBC connector(Java 侧)

**文件**: `connector/src/main/java/io/aqua/connector/dialects/DmDialect.java`

```java
public class DmDialect implements Dialect {
    @Override
    public String name() {
        return "dm";
    }

    @Override
    public List<String> listTables(Connection conn, String schema) throws SQLException {
        // 查询达梦系统表
        String sql = "SELECT TABLE_NAME FROM USER_TABLES WHERE OWNER = ?";
        // ...
    }

    @Override
    public List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException {
        // 反解达梦物理类型 -> aqua 逻辑类型
        // ...
    }
}
```

然后在 `registry.json` 注册:

```json
{
  "dialects": {
    "dm": "io.aqua.connector.dialects.DmDialect"
  }
}
```

---

## 方式二: 内置 Native 方言(高性能,需 Rust 驱动)

**适用**: MySQL/PostgreSQL 等有成熟 Rust 异步驱动的数据库。

### 1. 添加到 Dialect enum

**文件**: `crates/aqua-core/src/generators/ddl/types.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Dialect {
    #[default]
    Mysql,
    Postgresql,
    Sqlite,  // 新增 SQLite 内置支持
    Jdbc { name: String },
}
```

### 2. 实现类型映射

```rust
fn map_type(...) -> String {
    match dialect {
        Dialect::Mysql => map_mysql(...),
        Dialect::Postgresql => map_postgresql(...),
        Dialect::Sqlite => map_sqlite(...),  // 新增
        Dialect::Jdbc { name } => ...
    }
}

fn map_sqlite(data_type: DataType, ...) -> String {
    match data_type {
        DataType::Varchar => "TEXT".to_string(),  // SQLite 无 VARCHAR
        DataType::Long => "INTEGER".to_string(),
        // ...
    }
}
```

### 3. 实现 Driver trait

**文件**: `crates/aqua-core/src/driver/sqlite.rs`

```rust
use crate::driver::Driver;
use rusqlite::{Connection, Result};

pub struct SqliteDriver {
    conn: Connection,
}

impl Driver for SqliteDriver {
    async fn test_connection(&self) -> Result<()> {
        // SQLite 总是成功(文件不存在会自动创建)
        Ok(())
    }

    async fn list_tables(&self, _schema: &str) -> Result<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table'";
        // ...
    }

    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>> {
        let sql = format!("PRAGMA table_info({})", table);
        // 反解 SQLite 类型 -> aqua 逻辑类型
        // ...
    }
}
```

### 4. 添加依赖

**文件**: `crates/aqua-core/Cargo.toml`

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"], optional = true }

[features]
sqlite = ["rusqlite"]
```

---

## 新增方言 Checklist

- [ ] DDL 生成器类型映射(`generators/ddl/types.rs`)
- [ ] COMMENT 语法配置(`generators/ddl/table.rs`)
- [ ] DDL 生成器测试(`tests/generators_ddl.rs`)
- [ ] (JDBC 方言) connector.jar Dialect 实现
- [ ] (JDBC 方言) registry.json 注册
- [ ] (Native 方言) Driver trait 实现
- [ ] (Native 方言) Cargo.toml 依赖
- [ ] 更新文档(`docs/design.md` §4.1 类型映射表)

---

## 类型映射参考

aqua 定义了 9 种逻辑类型,每种数据库需提供物理类型映射:

| aqua 逻辑类型 | MySQL | PostgreSQL | Oracle | 达梦 DM | SQLite |
|--------------|-------|------------|--------|---------|--------|
| VARCHAR      | VARCHAR(n) | VARCHAR(n) | VARCHAR2(n) | VARCHAR(n) | TEXT |
| CLOB         | TEXT  | TEXT       | CLOB   | CLOB    | TEXT |
| TINYINT      | TINYINT | SMALLINT | NUMBER(3) | TINYINT | INTEGER |
| INT          | INT   | INTEGER    | NUMBER(10) | INT   | INTEGER |
| LONG         | BIGINT | BIGINT    | NUMBER(19) | BIGINT | INTEGER |
| DECIMAL      | DECIMAL(p,s) | NUMERIC(p,s) | NUMBER(p,s) | DECIMAL(p,s) | REAL |
| DATE         | DATE  | DATE       | DATE   | DATE    | TEXT |
| DATETIME     | DATETIME | TIMESTAMP | TIMESTAMP | TIMESTAMP | TEXT |
| BLOB         | BLOB  | BYTEA      | BLOB   | BLOB    | BLOB |

---

## 示例: 完整的 JDBC 方言实现

参考文件:
- Rust 侧: `crates/aqua-core/src/generators/ddl/types.rs` 中 `map_oracle` / `map_h2`
- Java 侧: `connector/src/main/java/io/aqua/connector/dialects/` (待实现)

---

## 常见问题

**Q: JDBC 方言的类型映射是否可以动态加载?**  
A: 当前硬编码在 Rust 侧,未来可通过 connector.jar 返回类型映射 JSON,Rust 侧动态查询。

**Q: 新增 Native 方言是否值得?**  
A: 仅当数据库有成熟的 Rust 异步驱动(如 tokio-postgres)且性能要求高时才推荐。大部分情况 JDBC 方言即可。

**Q: 如何测试新方言?**  
A: 编写单元测试验证类型映射,集成测试需真实数据库(Docker 启动或 CI 配置)。
