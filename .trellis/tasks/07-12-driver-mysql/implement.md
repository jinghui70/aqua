# driver-mysql 实现计划

## 实施步骤

1. [ ] `driver/mysql.rs` - MysqlDriver 结构体
2. [ ] 实现 test_connection
3. [ ] 实现 list_tables
4. [ ] 实现 get_columns + 类型反解
5. [ ] 实现 list_indexes
6. [ ] 更新 factory.rs
7. [ ] 单元测试(类型反解)

## MysqlDriver 结构

```rust
pub struct MysqlDriver {
    pool: mysql_async::Pool,
}

impl MysqlDriver {
    pub fn new(config: &DbConfig) -> Result<Self> {
        let opts = mysql_async::OptsBuilder::default()
            .ip_or_hostname(&config.host)
            .tcp_port(config.port)
            .user(Some(&config.user))
            .pass(Some(&config.password))
            .db_name(Some(&config.database));
        
        let pool = mysql_async::Pool::new(opts.into());
        Ok(Self { pool })
    }
}
```

## MySQL 类型反解

```rust
fn map_mysql_type(column_type: &str, length: Option<u32>) -> DataType {
    match column_type.to_uppercase().as_str() {
        "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
            if length.map_or(false, |l| l > 255) {
                DataType::Clob
            } else {
                DataType::Varchar
            }
        }
        "TINYINT" => DataType::Tinyint,
        "INT" | "INTEGER" | "MEDIUMINT" | "SMALLINT" => DataType::Int,
        "BIGINT" => DataType::Long,
        "DECIMAL" | "NUMERIC" => DataType::Decimal,
        "DATE" => DataType::Date,
        "DATETIME" | "TIMESTAMP" => DataType::Datetime,
        "BLOB" | "BINARY" | "VARBINARY" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" => DataType::Blob,
        _ => DataType::Varchar, // 默认
    }
}
```

## SQL 查询

### list_tables
```sql
SELECT TABLE_NAME 
FROM information_schema.TABLES 
WHERE TABLE_SCHEMA = ?
ORDER BY TABLE_NAME
```

### get_columns
```sql
SELECT 
    COLUMN_NAME,
    DATA_TYPE,
    CHARACTER_MAXIMUM_LENGTH,
    NUMERIC_PRECISION,
    NUMERIC_SCALE,
    IS_NULLABLE,
    COLUMN_KEY,
    COLUMN_DEFAULT,
    COLUMN_COMMENT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
ORDER BY ORDINAL_POSITION
```

### list_indexes
```sql
SELECT 
    INDEX_NAME,
    COLUMN_NAME,
    NON_UNIQUE
FROM information_schema.STATISTICS
WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
ORDER BY INDEX_NAME, SEQ_IN_INDEX
```

## 错误处理

```rust
impl From<mysql_async::Error> for DriverError {
    fn from(err: mysql_async::Error) -> Self {
        DriverError::QueryFailed(err.to_string())
    }
}
```

## 下一步

完成后更新 factory.rs:
```rust
pub fn create_driver(config: DbConfig) -> Result<Box<dyn Driver>> {
    match config.dialect.as_str() {
        "mysql" => Ok(Box::new(MysqlDriver::new(&config)?)),
        _ => Err(DriverError::UnsupportedDialect(config.dialect)),
    }
}
```
