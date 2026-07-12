//! ALTER 生成器集成测试。

use aqua_core::alter::{generate_alter, AlterOptions};
use aqua_core::diff::diff_project;
use aqua_core::generators::ddl::types::Dialect;
use aqua_core::schema::{parse_project, DataType, Project};
use std::fs;

fn load_fixture(name: &str) -> Project {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let json_str = fs::read_to_string(&path).expect("读取 fixture 失败");
    let value: serde_json::Value = serde_json::from_str(&json_str).expect("JSON 解析失败");
    parse_project(value).expect("Project 校验失败")
}

#[test]
fn test_alter_no_changes() {
    let project = load_fixture("valid-full.json");
    let diff = diff_project(&project, &project);
    let result = generate_alter(&diff, &project, &AlterOptions::default());
    assert!(result.is_empty(), "无差异时 ALTER 应为空");
}

#[test]
fn test_alter_add_field_mysql() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();

    let table = new
        .tables
        .iter_mut()
        .find(|t| t.code == "SYS_USER")
        .unwrap();
    table.fields.push(aqua_core::schema::Field {
        prop: "email".to_string(),
        code: "EMAIL".to_string(),
        name: "邮箱".to_string(),
        data_type: DataType::Varchar,
        length: Some(128),
        precision: None,
        scale: None,
        biz_type: None,
        biz_type_data: None,
        is_key: None,
        not_null: None,
        auto_generate: None,
        default_value: None,
        enum_ref: None,
        comment: None,
    });

    let diff = diff_project(&old, &new);
    let result = generate_alter(
        &diff,
        &new,
        &AlterOptions {
            dialect: Dialect::Mysql,
        },
    );

    println!("\n=== ALTER (MySQL) ===\n{}\n", result);

    assert!(
        result.contains("ALTER TABLE SYS_USER ADD COLUMN EMAIL VARCHAR(128)"),
        "应包含 ADD COLUMN"
    );
}

#[test]
fn test_alter_drop_field() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();

    let table = new
        .tables
        .iter_mut()
        .find(|t| t.code == "SYS_USER")
        .unwrap();
    table.fields.retain(|f| f.code != "REMARK");

    let diff = diff_project(&old, &new);
    let result = generate_alter(&diff, &new, &AlterOptions::default());

    assert!(
        result.contains("ALTER TABLE SYS_USER DROP COLUMN REMARK"),
        "应包含 DROP COLUMN"
    );
}

#[test]
fn test_alter_modify_field_dialects() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();

    let table = new
        .tables
        .iter_mut()
        .find(|t| t.code == "SYS_USER")
        .unwrap();
    let field = table
        .fields
        .iter_mut()
        .find(|f| f.code == "USER_NAME")
        .unwrap();
    field.length = Some(128);

    let diff = diff_project(&old, &new);

    // MySQL: MODIFY COLUMN
    let mysql = generate_alter(
        &diff,
        &new,
        &AlterOptions {
            dialect: Dialect::Mysql,
        },
    );
    assert!(mysql.contains("MODIFY COLUMN USER_NAME VARCHAR(128)"));

    // PostgreSQL: ALTER COLUMN TYPE
    let pg = generate_alter(
        &diff,
        &new,
        &AlterOptions {
            dialect: Dialect::Postgresql,
        },
    );
    assert!(pg.contains("ALTER COLUMN USER_NAME TYPE VARCHAR(128)"));

    // Oracle: MODIFY (...)
    let oracle = generate_alter(
        &diff,
        &new,
        &AlterOptions {
            dialect: Dialect::Jdbc {
                name: "oracle".to_string(),
            },
        },
    );
    assert!(oracle.contains("MODIFY (USER_NAME VARCHAR2(128)"));
}

#[test]
fn test_alter_drop_table() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();
    new.tables.clear();

    let diff = diff_project(&old, &new);
    let result = generate_alter(&diff, &new, &AlterOptions::default());

    assert!(result.contains("DROP TABLE SYS_USER;"), "应包含 DROP TABLE");
}
