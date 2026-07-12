//! diff 引擎集成测试。

use aqua_core::diff::diff_project;
use aqua_core::schema::{parse_project, Project};
use std::fs;

fn load_fixture(name: &str) -> Project {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    let json_str = fs::read_to_string(&path).expect("读取 fixture 失败");
    let value: serde_json::Value = serde_json::from_str(&json_str).expect("JSON 解析失败");
    parse_project(value).expect("Project 校验失败")
}

#[test]
fn test_diff_identical_projects() {
    let old = load_fixture("valid-full.json");
    let new = load_fixture("valid-full.json");
    let result = diff_project(&old, &new);

    assert!(result.tables.added.is_empty(), "相同项目不应有新增表");
    assert!(result.tables.removed.is_empty(), "相同项目不应有删除表");
    assert!(result.tables.changed.is_empty(), "相同项目不应有变更表");
}

#[test]
fn test_diff_added_table() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();
    new.tables.push(aqua_core::schema::Table {
        code: "SYS_NEW_TABLE".to_string(),
        name: "新表".to_string(),
        group: "core".to_string(),
        fields: vec![],
        indexes: None,
        comment: None,
    });

    let result = diff_project(&old, &new);
    assert!(result.tables.added.contains(&"SYS_NEW_TABLE".to_string()));
    assert!(result.tables.removed.is_empty());
    assert!(result.tables.changed.is_empty());
}

#[test]
fn test_diff_removed_table() {
    let old = load_fixture("valid-full.json");
    let new = {
        let mut p = old.clone();
        p.tables.clear();
        p
    };

    let result = diff_project(&old, &new);
    assert!(result.tables.removed.contains(&"SYS_USER".to_string()));
    assert!(result.tables.added.is_empty());
}

#[test]
fn test_diff_changed_field() {
    let old = load_fixture("valid-full.json");
    let mut new = old.clone();

    // 修改 USER_NAME 字段的长度
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
    field.length = Some(128); // 原 64 -> 128

    let result = diff_project(&old, &new);
    assert!(!result.tables.changed.is_empty(), "应有表变更");

    let table_change = result
        .tables
        .changed
        .iter()
        .find(|c| c.table == "SYS_USER")
        .expect("SYS_USER 应在变更列表");

    let field_change = table_change
        .fields
        .changed
        .iter()
        .find(|fc| fc.field == "USER_NAME")
        .expect("USER_NAME 应在字段变更列表");
    assert!(
        field_change.changes.contains(&"length".to_string()),
        "应检测到 length 变更"
    );
}

#[test]
fn test_diff_added_field() {
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
        data_type: aqua_core::schema::DataType::Varchar,
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

    let result = diff_project(&old, &new);
    let table_change = result
        .tables
        .changed
        .iter()
        .find(|c| c.table == "SYS_USER")
        .expect("应有 SYS_USER 变更");
    assert!(
        table_change.fields.added.contains(&"EMAIL".to_string()),
        "应检测到新增字段 EMAIL"
    );
}
