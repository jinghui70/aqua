# driver-trait 实现计划

## 实施步骤

1. [ ] `driver/mod.rs` - 模块声明 + pub use
2. [ ] `driver/types.rs` - DbConfig/ColumnMeta/IndexMeta
3. [ ] `driver/error.rs` - DriverError 错误类型
4. [ ] `driver/trait_def.rs` - Driver trait 定义
5. [ ] `driver/factory.rs` - create_driver 工厂(占位实现)
6. [ ] 更新 `lib.rs` - pub mod driver

## 类型设计

### DbConfig
```rust
pub struct DbConfig {
    pub dialect: String,    // "mysql" | "postgresql" | "oracle" | ...
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub schema: Option<String>,  // 部分数据库需要
}
```

### ColumnMeta
```rust
pub struct ColumnMeta {
    pub name: String,
    pub data_type: DataType,  // aqua 逻辑类型
    pub length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    pub nullable: bool,
    pub is_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}
```

### IndexMeta
```rust
pub struct IndexMeta {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}
```

## Driver trait

```rust
#[async_trait]
pub trait Driver {
    async fn test_connection(&self) -> Result<(), DriverError>;
    async fn list_tables(&self, schema: &str) -> Result<Vec<String>, DriverError>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, DriverError>;
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>, DriverError>;
    // async fn query_rows(&self, sql: &str, limit: usize) -> Result<Vec<Row>, DriverError>;
}
```

## 工厂模式

```rust
pub fn create_driver(config: DbConfig) -> Result<Box<dyn Driver>, DriverError> {
    match config.dialect.as_str() {
        "mysql" => todo!("MySQL native 驱动,后续任务实现"),
        "postgresql" | "postgres" => todo!("PostgreSQL native 驱动"),
        _ => todo!("JDBC 驱动,spawn connector.jar"),
    }
}
```

## 依赖

- `async-trait` - async trait 支持
- `thiserror` - 错误类型派生(已有)

## 下一步

本任务只定义接口,不实现具体驱动。后续任务:
- 07-12-driver-mysql: MySQL native 实现
- 07-12-driver-jdbc: JDBC 驱动实现
