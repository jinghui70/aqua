//! schema 模块集成测试 - 移植自 legacy `__tests__/schema.test.ts`。

use aqua_core::schema::{parse_project, validate_project, ParseError, Project};
use std::fs;
use std::path::PathBuf;

/// 加载 fixture JSON 文件。
fn load_fixture(name: &str) -> serde_json::Value {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    let content =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("无法读取 fixture: {:?}", path));
    serde_json::from_str(&content).expect("fixture JSON 解析失败")
}

/// 辅助函数: validateProject 等价(返回 Result)。
fn validate_project_result(
    value: serde_json::Value,
) -> Result<Project, Vec<aqua_core::schema::ValidationError>> {
    let project: Project = serde_json::from_value(value).expect("serde 反序列化应成功");
    validate_project(&project)?;
    Ok(project)
}

#[test]
fn test_accepts_complete_valid_project() {
    let result = validate_project_result(load_fixture("valid-full.json"));
    assert!(result.is_ok(), "valid-full 应通过校验");

    let project = result.unwrap();
    assert_eq!(project.base_package, "com.example");
    assert_eq!(project.tables[0].code, "SYS_USER");
    assert_eq!(project.tables[0].fields.len(), 7);
    assert_eq!(project.enums.len(), 2);
}

#[test]
fn test_parse_project_returns_parsed_project_for_valid_input() {
    let project =
        parse_project(load_fixture("valid-full.json")).expect("parseProject 应成功返回 Project");
    assert_eq!(project.version, "1.0.0");
    assert!(project.tables[0].indexes.as_ref().unwrap()[0].unique);
}

#[test]
fn test_rejects_enum_on_non_varchar_datatype() {
    let result = validate_project_result(load_fixture("invalid-enum-non-varchar.json"));
    assert!(result.is_err(), "enum 非 VARCHAR 应失败");

    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("enum 只支持 VARCHAR")),
        "应包含 'enum 只支持 VARCHAR' 错误"
    );
}

#[test]
fn test_rejects_has_code_true_when_value_missing_code() {
    let result = validate_project_result(load_fixture("invalid-has-code-missing-code.json"));
    assert!(result.is_err(), "hasCode=true 缺 code 应失败");

    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.message.contains("hasCode")),
        "应包含 hasCode 相关错误"
    );
}

#[test]
fn test_rejects_missing_required_fields() {
    // serde 短路路线: 缺失必填字段在反序列化阶段就失败(只报第一个缺失字段)。
    // 调整断言: 验证 parse_project 抛出 Deserialize 错误,且消息包含缺失字段名。
    let result = parse_project(load_fixture("invalid-missing-required.json"));
    assert!(result.is_err(), "缺失必填字段应失败");

    match result.unwrap_err() {
        ParseError::Deserialize(e) => {
            let msg = e.to_string();
            // serde 会报第一个遇到的缺失字段(group 或 code,具体顺序由 JSON 字段顺序决定)
            assert!(
                msg.contains("group") || msg.contains("code"),
                "错误消息应包含缺失字段名: {}",
                msg
            );
        }
        ParseError::Validate(_) => panic!("应是 Deserialize 错误,不是 Validate"),
    }
}

#[test]
fn test_parse_project_throws_on_invalid_input() {
    // parseProject 遇到校验错误应返回 ParseError::Validate。
    let result = parse_project(load_fixture("invalid-enum-non-varchar.json"));
    assert!(result.is_err());

    match result.unwrap_err() {
        ParseError::Validate(errors) => {
            assert!(!errors.is_empty(), "应有校验错误");
        }
        ParseError::Deserialize(_) => panic!("此 fixture 应通过反序列化,失败在校验层"),
    }
}

#[test]
fn test_rejects_empty_values_array() {
    // 手写 project: enums[0].values = []
    let project_json = serde_json::json!({
        "version": "1.0.0",
        "basePackage": "com.example",
        "bizTypes": [],
        "enums": [{
            "code": "E",
            "name": "E",
            "package": "enum",
            "values": []
        }],
        "groups": [],
        "tables": []
    });

    let result = validate_project_result(project_json);
    assert!(result.is_err(), "空 values 数组应失败");

    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.path.contains("values")),
        "应包含 values 相关错误"
    );
}

#[test]
fn test_serde_roundtrip() {
    // 新增测试: valid-full JSON -> Project -> JSON,验证往返等价。
    let original_json = load_fixture("valid-full.json");
    let project: Project = serde_json::from_value(original_json.clone()).expect("反序列化应成功");
    let serialized_json = serde_json::to_value(&project).expect("序列化应成功");

    // 比较两个 JSON Value(serde_json::Value 实现 PartialEq)
    assert_eq!(original_json, serialized_json, "serde 往返应保持 JSON 等价");
}
