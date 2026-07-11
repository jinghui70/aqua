# Database Guidelines

> Database patterns and conventions for aqua-core.

---

## Overview

aqua-core 是**数据库设计工具**,不是应用后端,自身不持有业务数据库。它的"数据库"交互分两类:

1. **连接目标库**(用户待导入的 MySQL/Oracle/PG 等) - 通过 `Driver trait` 读元数据
2. **dataset SQLite 载体**(schema.json + 数据集打包为 .aqua 文件) - 用 rusqlite 读写

aqua-core 无 ORM / 无 migration(目标库元数据只读,dataset 是临时容器)。

---

## Query Patterns

### 目标库元数据查询(通过 Driver trait)

```rust
// Driver trait 统一接口(native + JDBC 两实现)
pub trait Driver {
    async fn test_connection(&self) -> Result<()>;
    async fn list_tables(&self, schema: &str) -> Result<Vec<String>>;
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMeta>>;
    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexMeta>>;
    async fn query_rows(&self, sql: &str, limit: usize) -> Result<Vec<Row>>;
}
```

**原则**:
- **只读元数据**: 不写目标库(DDL 生成是文本输出,不直接执行)
- **一次性连接**: native 驱动不维护连接池(aqua 是低频设计工具,冷启动可接受)
- **统一返回**: 各驱动内部反解物理类型 → 返回统一的 aqua schema 类型(`DataType` / `Field` / `Table`)

### dataset SQLite 读写(rusqlite)

```rust
use rusqlite::{Connection, params};

// 创建 dataset 容器
let conn = Connection::open("project.aqua")?;
conn.execute(
    "CREATE TABLE IF NOT EXISTS _aqua_meta (key TEXT PRIMARY KEY, value TEXT)",
    [],
)?;

// 存 schema.json
conn.execute(
    "INSERT INTO _aqua_meta (key, value) VALUES (?1, ?2)",
    params!["schema", &schema_json],
)?;

// 存数据集表(用户数据)
conn.execute(&format!("CREATE TABLE {} (...)", table_code), [])?;
```

**原则**:
- **嵌入式 SQLite**: `rusqlite` + `features = ["bundled"]`,无外部依赖
- **单文件**: `.aqua` = SQLite 文件,内含 schema.json + 数据表
- **事务包裹**: 批量写入用 `conn.transaction()` 确保原子性

---

## Migrations

aqua-core 自身**无 migration**:

- **目标库**: 只读元数据,不执行 ALTER(DDL 生成是文本输出,用户自行执行)
- **dataset SQLite**: 临时容器,随 schema.json 版本字段演进,不做向后兼容 migration(旧版 .aqua 文件用旧版 aqua 打开,或提示升级)

---

## Naming Conventions

### 目标库反解(读取已有命名)

- **表名/字段名/索引名**: 保持目标库原样(MySQL 大写 / PG 小写 / Oracle 大写),存入 schema.json 的 `code` 字段
- **prop 生成**: `code` → `prop`(蛇形→小驼峰,如 `USER_NAME` → `userName`),在 UI 层生成,不在 aqua-core

### dataset SQLite 内部表

- **元数据表**: `_aqua_meta`(前缀 `_aqua_` 保留)
- **数据集表**: 直接用 schema.json 的 `table.code`(如 `SYS_USER`),保持与源库一致

---

## Connection Management

### Native 驱动(MySQL/PG)

```rust
// MySQL: mysql_async,一次性连接不维护池
let pool = mysql_async::Pool::new("mysql://...");
let mut conn = pool.get_conn().await?;
let tables: Vec<String> = conn.query("SHOW TABLES").await?;
drop(conn);  // 用完即关

// PG: tokio-postgres + deadpool(临时池,仅本次操作生存期)
let pool = deadpool_postgres::Pool::new(...);
let client = pool.get().await?;
let rows = client.query("SELECT * FROM information_schema.tables", &[]).await?;
```

**原则**: 低频工具,无需常驻池,每次 `Driver` 操作独立连接生命周期。

### JDBC 驱动(Oracle/信创)

通过 `connector.jar` 一次性命令:

```bash
echo '{"action":"listTables","schema":"SCOTT"}' | java -jar connector.jar
# stdout: {"tables":["EMP","DEPT"]}
```

aqua-core 用 `tokio::process::Command` spawn,stdin/stdout JSON 通信,进程结束即连接关闭。

---

## Error Handling

### 连接失败

```rust
match driver.test_connection().await {
    Ok(_) => { /* 连接成功 */ },
    Err(e) => {
        // 返回用户友好错误(含 host/port/user,隐藏密码)
        return Err(format!("连接失败: {}@{} - {}", config.user, config.host, e));
    }
}
```

### SQL 错误

```rust
// 元数据查询失败(如表不存在/权限不足)
match driver.get_columns("UNKNOWN_TABLE").await {
    Err(e) if e.to_string().contains("doesn't exist") => {
        return Err("表不存在".into());
    }
    Err(e) => return Err(format!("查询失败: {}", e)),
    Ok(cols) => cols,
}
```

**原则**: 数据库错误转为用户可理解的消息,隐藏密码/敏感信息。

---

## Common Mistakes

### ❌ 在 aqua-core 里直接执行 DDL

```rust
conn.execute(&ddl_sql, [])?;  // ❌ aqua-core 只生成 DDL 文本,不执行
```

**✅ 正确**: DDL 输出到文件,用户自行在目标库执行。

### ❌ 混淆目标库与 dataset SQLite

```rust
// ❌ 把 schema.json 的表名当成目标库查询
let rows = mysql_conn.query("SELECT * FROM SYS_USER").await?;  // 可能不存在
```

**✅ 正确**: 
- 导入时从目标库读元数据 → 生成 schema.json
- dataset 是 schema.json + 数据的打包容器,两者职责分离

### ❌ native 驱动维护长连接池

```rust
lazy_static! {
    static ref POOL: Pool = Pool::new(...);  // ❌ aqua 是低频工具,无需全局池
}
```

**✅ 正确**: 每次操作临时连接,用完即关,简化生命周期管理。

---

## Security

- **密码加密**: `DataSource` 的 password 字段用 AES-256-GCM 加密存储(见 design.md §6)
- **SQL 注入**: 元数据查询用参数化(如 PG `$1` / MySQL `?`),表名/字段名用白名单校验
- **日志脱敏**: 连接错误日志隐藏密码,仅显示 `user@host:port`

---

## Examples

**参考实现**: 
- Driver trait 定义: `crates/aqua-core/src/driver/mod.rs`(待移植)
- MySQL native: `crates/aqua-core/src/driver/mysql.rs`(待移植)
- dataset SQLite: `crates/aqua-core/src/dataset/mod.rs`(待移植)
