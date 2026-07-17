//! 数据库集成测试(MySQL/PG 真实库)。
//!
//! ## 运行方式
//!
//! 测试标记 `#[ignore]`,默认 `cargo test` 不执行(避免无 Docker 环境失败)。
//!
//! 手动运行:
//! ```sh
//! # 1. 启动数据库容器
//! docker compose up -d
//!
//! # 2. 等待健康检查通过(约 10-30s)
//! docker compose ps   # 确认 mysql/postgres 为 healthy
//!
//! # 3. 运行集成测试
//! cargo test -p aqua-core --test integration_db -- --ignored
//!
//! # 4. 清理
//! docker compose down
//! ```
//!
//! ## 测试覆盖
//!
//! - Driver trait: test_connection / list_tables / get_columns / list_indexes
//! - import_from_db 全链路: DDL 建表 -> 导入 -> 比对 Project

use aqua_core::driver::{create_driver, DbConfig};
use aqua_core::generators::ddl::types::Dialect;
use aqua_core::generators::ddl::{generate_ddl, DdlOptions};
use aqua_core::import::import_from_db;
use aqua_core::schema::{parse_project, Project};
use std::fs;

/// 加载 valid-full fixture 作为建表 + 比对基准。
fn load_fixture() -> Project {
    let path = format!(
        "{}/tests/fixtures/valid-full.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json_str = fs::read_to_string(&path).expect("读取 fixture 失败");
    let value: serde_json::Value = serde_json::from_str(&json_str).expect("JSON 解析失败");
    parse_project(value).expect("Project 校验失败")
}

/// MySQL 配置(对齐 docker-compose.yml)。
fn mysql_config() -> DbConfig {
    DbConfig {
        dialect: "mysql".to_string(),
        host: "localhost".to_string(),
        port: 3306,
        user: "root".to_string(),
        password: "root".to_string(),
        database: "aqua_test".to_string(),
        schema: None,
    }
}

/// PG 配置(对齐 docker-compose.yml)。
fn pg_config() -> DbConfig {
    DbConfig {
        dialect: "postgresql".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        user: "postgres".to_string(),
        password: "root".to_string(),
        database: "aqua_test".to_string(),
        schema: Some("public".to_string()),
    }
}

// ============================================================
// MySQL 集成测试
// ============================================================

#[tokio::test]
#[ignore]
async fn mysql_test_connection() {
    let driver = create_driver(mysql_config(), None, "connector.jar").expect("创建 MySQL 驱动失败");
    driver.test_connection().await.expect("MySQL 连接应成功");
}

#[tokio::test]
#[ignore]
async fn mysql_full_roundtrip() {
    let config = mysql_config();
    let project = load_fixture();

    // 1. 用 mysql_async 建表(测试准备,非 Driver 职责)
    let opts = mysql_async::OptsBuilder::default()
        .ip_or_hostname(&config.host)
        .tcp_port(config.port)
        .user(Some(&config.user))
        .pass(Some(&config.password))
        .db_name(Some(&config.database));
    let pool = mysql_async::Pool::new(mysql_async::Opts::from(opts));
    {
        use mysql_async::prelude::Queryable;
        let mut conn = pool.get_conn().await.expect("获取连接失败");
        // 清理旧表
        let _ = conn.query_drop("DROP TABLE IF EXISTS SYS_USER").await;
        // 生成并执行 DDL
        let ddl = generate_ddl(
            &project,
            &DdlOptions {
                dialect: Dialect::Mysql,
                ..Default::default()
            },
        );
        for stmt in ddl.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            conn.query_drop(stmt).await.expect("执行 DDL 失败");
        }
    }
    pool.disconnect().await.expect("断开连接失败");

    // 2. Driver: test_connection
    let driver = create_driver(config.clone(), None, "connector.jar").expect("创建驱动失败");
    driver.test_connection().await.expect("连接应成功");

    // 3. Driver: list_tables
    let tables = driver
        .list_tables(&config.database)
        .await
        .expect("list_tables 失败");
    assert!(
        tables.iter().any(|t| t == "SYS_USER"),
        "应包含 SYS_USER 表,实际: {:?}",
        tables
    );

    // 4. Driver: get_columns
    let columns = driver
        .get_columns("SYS_USER")
        .await
        .expect("get_columns 失败");
    assert!(!columns.is_empty(), "应有列");
    assert!(
        columns.iter().any(|c| c.name == "ID"),
        "应包含 ID 列,实际: {:?}",
        columns.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    // ID 是主键
    let id_col = columns.iter().find(|c| c.name == "ID").unwrap();
    assert!(id_col.is_key, "ID 应为主键");

    // 5. Driver: list_indexes
    let indexes = driver
        .list_indexes("SYS_USER")
        .await
        .expect("list_indexes 失败");
    assert!(indexes.iter().any(|i| i.unique), "应有唯一索引(USER_NAME)");

    // 6. import_from_db 全链路
    let driver2 = create_driver(config.clone(), None, "connector.jar").expect("创建导入驱动失败");
    let imported = import_from_db(
        driver2.as_ref(),
        &["SYS_USER".to_string()],
        Some("com.example".to_string()),
    )
    .await
    .expect("导入失败");

    assert!(
        imported.tables.iter().any(|t| t.code == "SYS_USER"),
        "导入应含 SYS_USER"
    );
    let imported_table = imported
        .tables
        .iter()
        .find(|t| t.code == "SYS_USER")
        .unwrap();
    assert!(
        imported_table.fields.iter().any(|f| f.code == "ID"),
        "导入字段应含 ID"
    );
    assert!(
        imported_table
            .fields
            .iter()
            .find(|f| f.code == "ID")
            .unwrap()
            .is_key
            .unwrap_or(false),
        "导入 ID 应为主键"
    );
}

// ============================================================
// PostgreSQL 集成测试
// ============================================================

#[tokio::test]
#[ignore]
async fn pg_test_connection() {
    let driver = create_driver(pg_config(), None, "connector.jar").expect("创建 PG 驱动失败");
    driver.test_connection().await.expect("PG 连接应成功");
}

#[tokio::test]
#[ignore]
async fn pg_full_roundtrip() {
    let config = pg_config();
    let project = load_fixture();

    // 1. 用 deadpool_postgres 建表
    let mut cfg = deadpool_postgres::Config::new();
    cfg.host = Some(config.host.clone());
    cfg.port = Some(config.port);
    cfg.user = Some(config.user.clone());
    cfg.password = Some(config.password.clone());
    cfg.dbname = Some(config.database.clone());
    let pool = cfg
        .create_pool(None, tokio_postgres::NoTls)
        .expect("创建池失败");
    {
        let client = pool.get().await.expect("获取连接失败");
        client
            .execute("DROP TABLE IF EXISTS SYS_USER", &[])
            .await
            .ok();
        let ddl = generate_ddl(
            &project,
            &DdlOptions {
                dialect: Dialect::Postgresql,
                ..Default::default()
            },
        );
        for stmt in ddl.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            client.execute(stmt, &[]).await.expect("执行 DDL 失败");
        }
    }

    // 2. Driver: list_tables
    let driver = create_driver(config.clone(), None, "connector.jar").expect("创建驱动失败");
    driver.test_connection().await.expect("连接应成功");

    let tables = driver
        .list_tables("public")
        .await
        .expect("list_tables 失败");
    assert!(
        tables.iter().any(|t| t.eq_ignore_ascii_case("sys_user")),
        "应包含 sys_user 表,实际: {:?}",
        tables
    );

    // 3. Driver: get_columns(PG 表名小写)
    let table_name = tables
        .iter()
        .find(|t| t.eq_ignore_ascii_case("sys_user"))
        .unwrap()
        .clone();
    let columns = driver
        .get_columns(&table_name)
        .await
        .expect("get_columns 失败");
    assert!(!columns.is_empty(), "应有列");

    // 4. import_from_db
    let driver2 = create_driver(config.clone(), None, "connector.jar").expect("创建导入驱动失败");
    let imported = import_from_db(driver2.as_ref(), &["SYS_USER".to_string()], Some("com.example".to_string()))
        .await
        .expect("导入失败");

    assert!(
        imported
            .tables
            .iter()
            .any(|t| t.code.eq_ignore_ascii_case("SYS_USER")),
        "导入应含 SYS_USER(导入时 to_uppercase)"
    );
}

// ============================================================
// JDBC (H2) 集成测试 - 验证 connector.jar 路径 + check_java + spawn 全链路
// ============================================================

/// H2 内存库配置(对齐 connector H2Dialect: host 空 -> jdbc:h2:mem:)。
fn h2_config() -> DbConfig {
    DbConfig {
        dialect: "h2".to_string(),
        host: "".to_string(),
        port: 0,
        user: "sa".to_string(),
        password: "".to_string(),
        database: "aqua_test".to_string(),
        schema: None,
    }
}

/// connector.jar 绝对路径(需先 `pnpm build:connector` 产出 src-tauri/resources/connector.jar)。
fn connector_jar() -> String {
    format!(
        "{}/../../src-tauri/resources/connector.jar",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[tokio::test]
#[ignore]
async fn h2_test_connection() {
    // 全链路:create_driver(connector_path) -> JdbcDriver::call -> check_java -> spawn connector.jar
    let driver = create_driver(h2_config(), None, &connector_jar()).expect("创建 H2 驱动失败");
    driver.test_connection().await.expect("H2 连接应成功");
}
