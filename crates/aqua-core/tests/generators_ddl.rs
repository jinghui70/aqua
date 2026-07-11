//! DDL 生成器集成测试。

use aqua_core::generators::ddl::{generate_ddl, DdlOptions, Dialect};
use aqua_core::schema::parse_project;
use std::fs;

/// 加载 fixture。
fn load_fixture(name: &str) -> aqua_core::schema::Project {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let json_str = fs::read_to_string(&path).expect("读取 fixture 失败");
    let value: serde_json::Value = serde_json::from_str(&json_str).expect("JSON 解析失败");
    parse_project(value).expect("Project 校验失败")
}

#[test]
fn test_mysql_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Mysql,
            ..Default::default()
        },
    );

    // 验证 CREATE TABLE
    assert!(ddl.contains("CREATE TABLE SYS_USER"), "应包含 SYS_USER 表");

    // 验证类型映射
    assert!(
        ddl.contains("BIGINT"),
        "应包含 BIGINT 类型 (LONG -> BIGINT)"
    );
    assert!(ddl.contains("VARCHAR(64)"), "应包含 VARCHAR 类型");
    assert!(ddl.contains("TEXT"), "应包含 TEXT 类型 (CLOB -> TEXT)");
    assert!(ddl.contains("DECIMAL(12, 2)"), "应包含 DECIMAL 类型");
    assert!(ddl.contains("DATETIME"), "应包含 DATETIME 类型");

    // 验证 PRIMARY KEY
    assert!(ddl.contains("PRIMARY KEY (ID)"), "应包含 PRIMARY KEY");

    // 验证 COMMENT (MySQL 内联)
    assert!(ddl.contains("COMMENT '用户'"), "应包含表注释");
    assert!(ddl.contains("COMMENT '主键'"), "应包含字段注释");

    // 验证 CREATE INDEX
    assert!(
        ddl.contains("CREATE UNIQUE INDEX IDX_SYS_USER_USER_NAME"),
        "应包含唯一索引"
    );
    assert!(ddl.contains("CREATE INDEX IDX_GENDER"), "应包含普通索引");
}

#[test]
fn test_postgresql_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Postgresql,
            ..Default::default()
        },
    );

    // 验证类型映射
    assert!(ddl.contains("BIGINT"), "应包含 BIGINT");
    assert!(ddl.contains("VARCHAR(64)"), "应包含 VARCHAR");
    assert!(ddl.contains("TEXT"), "应包含 TEXT");
    assert!(ddl.contains("NUMERIC(12, 2)"), "PG DECIMAL -> NUMERIC");
    assert!(ddl.contains("TIMESTAMP"), "PG DATETIME -> TIMESTAMP");

    // 验证 COMMENT (PG 独立语句)
    assert!(
        ddl.contains("COMMENT ON TABLE SYS_USER IS '用户'"),
        "应包含表注释"
    );
    assert!(
        ddl.contains("COMMENT ON COLUMN SYS_USER.ID IS '主键'"),
        "应包含列注释"
    );
}

#[test]
fn test_oracle_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Jdbc {
                name: "oracle".to_string(),
            },
            ..Default::default()
        },
    );

    // 验证 Oracle 特有类型
    assert!(ddl.contains("VARCHAR2(64)"), "Oracle VARCHAR -> VARCHAR2");
    assert!(ddl.contains("NUMBER(19)"), "Oracle LONG -> NUMBER(19)");
    assert!(ddl.contains("CLOB"), "Oracle CLOB");
}

#[test]
fn test_h2_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Jdbc {
                name: "h2".to_string(),
            },
            ..Default::default()
        },
    );

    // H2 类似 MySQL
    assert!(ddl.contains("VARCHAR(64)"));
    assert!(ddl.contains("BIGINT"));
    assert!(ddl.contains("CLOB"));
}

#[test]
fn test_filter_by_tables() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Mysql,
            tables: Some(vec!["SYS_USER".to_string()]),
            ..Default::default()
        },
    );

    // 只包含指定表
    assert!(ddl.contains("SYS_USER"), "应包含 SYS_USER");
    assert!(!ddl.is_empty(), "DDL 不应为空");
}

#[test]
fn test_empty_filter() {
    let project = load_fixture("valid-full.json");

    // tables 过滤为空列表
    let ddl = generate_ddl(
        &project,
        &DdlOptions {
            dialect: Dialect::Mysql,
            tables: Some(vec![]),
            ..Default::default()
        },
    );

    assert!(ddl.is_empty(), "空过滤应返回空 DDL");
}
