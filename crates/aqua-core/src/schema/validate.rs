//! 业务校验层 - 独立于 serde 的语义规则校验。

use crate::schema::data_type::DataType;
use crate::schema::keywords::{is_java_keyword, is_sql_reserved};
use crate::schema::project::Project;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// ValidationError - 带 path + message,对齐 legacy errors 结构,前端可定位字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// ParseError - 统一的解析错误类型。
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("JSON 反序列化失败: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("Project 校验失败: {count} 个错误", count = .0.len())]
    Validate(Vec<ValidationError>),
}

/// validate_project - 业务校验,收集所有错误(不短路,对齐 legacy 一次返回全部 errors)。
///
/// 校验规则(§3 数据模型 + 保存时合法性):
/// - 各类型只允许特定属性(VARCHAR length / DECIMAL precision,scale / 其余无)
/// - enum 只支持 VARCHAR (field.rs 规则)
/// - hasCode=true 时每个 value 必须有 code (enum_def.rs 规则)
/// - values 非空
/// - code 表内不重复 / code 非 SQL 保留字 / prop 非 Java 关键字
/// - DECIMAL precision >= scale
/// - 索引名表内不重复(空名不查,空名自动生成)
pub fn validate_project(project: &Project) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 校验表
    for table in project.tables.iter() {
        let mut seen_codes: HashSet<&str> = HashSet::new();
        let mut seen_props: HashSet<&str> = HashSet::new();

        // 校验字段
        for field in table.fields.iter() {
            let base = format!("{}.{}", table.code, field.code);

            // prop 不能为空
            if field.prop.is_empty() {
                errors.push(ValidationError::new(
                    format!("{}.prop", base),
                    "不能为空",
                ));
            }
            // name 不能为空
            if field.name.is_empty() {
                errors.push(ValidationError::new(
                    format!("{}.name", base),
                    "不能为空",
                ));
            }

            // code 表内重复(空 code 跳过,空由保存清理处理)
            if !field.code.is_empty() && !seen_codes.insert(field.code.as_str()) {
                errors.push(ValidationError::new(
                    format!("{}.code", base),
                    format!("重复: {}", field.code),
                ));
            }
            // prop 表内重复(空 prop 跳过)
            if !field.prop.is_empty() && !seen_props.insert(field.prop.as_str()) {
                errors.push(ValidationError::new(
                    format!("{}.prop", base),
                    format!("重复: {}", field.prop),
                ));
            }

            // code SQL 保留字
            if is_sql_reserved(&field.code) {
                errors.push(ValidationError::new(
                    format!("{}.code", base),
                    format!("code '{}' 是 SQL 保留字", field.code),
                ));
            }

            // prop Java 关键字
            if is_java_keyword(&field.prop) {
                errors.push(ValidationError::new(
                    format!("{}.prop", base),
                    format!("prop '{}' 是 Java 关键字", field.prop),
                ));
            }

            // 规则: §3.1 各类型只允许特定属性
            //   - VARCHAR: length
            //   - DECIMAL: precision, scale
            //   - 其余: 均无
            // 多余属性报错,避免脏数据流入生成器(DDL/Java/前端 JSON)。
            let (allow_length, allow_precision, allow_scale) = match field.data_type {
                DataType::Varchar => (true, false, false),
                DataType::Decimal => (false, true, true),
                _ => (false, false, false),
            };
            let dt_name = format!("{:?}", field.data_type).to_uppercase();
            if !allow_length && field.length.is_some() {
                errors.push(ValidationError::new(
                    format!("{}.length", base),
                    format!("{} 不允许 length", dt_name),
                ));
            }
            if !allow_precision && field.precision.is_some() {
                errors.push(ValidationError::new(
                    format!("{}.precision", base),
                    format!("{} 不允许 precision", dt_name),
                ));
            }
            if !allow_scale && field.scale.is_some() {
                errors.push(ValidationError::new(
                    format!("{}.scale", base),
                    format!("{} 不允许 scale", dt_name),
                ));
            }

            // VARCHAR length 不能为空; DECIMAL precision/scale 不能为空
            if field.data_type == DataType::Varchar && field.length.is_none() {
                errors.push(ValidationError::new(
                    format!("{}.length", base),
                    "不能为空",
                ));
            }
            if field.data_type == DataType::Decimal {
                if field.precision.is_none() {
                    errors.push(ValidationError::new(
                        format!("{}.precision", base),
                        "不能为空",
                    ));
                }
                if field.scale.is_none() {
                    errors.push(ValidationError::new(
                        format!("{}.scale", base),
                        "不能为空",
                    ));
                }
            }

            // DECIMAL precision >= scale(两者都 Some 时)
            if field.data_type == DataType::Decimal {
                if let (Some(p), Some(s)) = (field.precision, field.scale) {
                    if p < s {
                        errors.push(ValidationError::new(
                            format!("{}.precision", base),
                            format!("precision({}) 不能小于 scale({})", p, s),
                        ));
                    }
                }
            }

            // 规则: enum 只支持 VARCHAR
            if field.enum_ref.is_some() && field.data_type != DataType::Varchar {
                errors.push(ValidationError::new(
                    format!("{}.enum", base),
                    format!("enum 只支持 VARCHAR，当前 dataType={:?}", field.data_type),
                ));
            }

            // 校验内联枚举
            if let Some(inline_enum) = &field.enum_ref {
                // values 非空
                if inline_enum.values.is_empty() {
                    errors.push(ValidationError::new(
                        format!("{}.enum.values", base),
                        "values 数组不能为空",
                    ));
                }

                // hasCode=true 时每个 value 必须有 code
                if inline_enum.has_code.unwrap_or(false) {
                    for (value_idx, value) in inline_enum.values.iter().enumerate() {
                        if value.code.is_none() || value.code.as_ref().unwrap().is_empty() {
                            errors.push(ValidationError::new(
                                format!("{}.enum.values[{}].code", base, value_idx),
                                "hasCode=true 时每个 value 必须有 code",
                            ));
                        }
                    }
                }
            }
        }

        // 索引: 名重复(空名不查) + 索引重复(fields 序列 + unique 完全相同)
        if let Some(indexes) = &table.indexes {
            let mut seen_names: HashSet<String> = HashSet::new();
            let mut seen_index: HashSet<(Vec<(String, &str)>, bool)> = HashSet::new();
            for idx in indexes.iter() {
                // 索引名不能为空(不自动生成,用户必填)
                if idx.name.as_deref().map(|n| n.is_empty()).unwrap_or(true) {
                    errors.push(ValidationError::new(
                        format!("{}.[{}]", table.code, idx.name.as_deref().unwrap_or("")),
                        "索引名不能为空",
                    ));
                }
                if let Some(name) = &idx.name {
                    if !name.is_empty() && !seen_names.insert(name.clone()) {
                        errors.push(ValidationError::new(
                            format!("{}.[{}]", table.code, name),
                            "索引名重复".to_string(),
                        ));
                    }
                }
                let key: Vec<(String, &str)> = idx
                    .fields
                    .iter()
                    .map(|f| (f.code.clone(), f.direction.as_str()))
                    .collect();
                if !seen_index.insert((key, idx.unique)) {
                    let label = idx.name.as_deref().unwrap_or("自动命名");
                    errors.push(ValidationError::new(
                        format!("{}.[{}]", table.code, label),
                        "索引重复(字段与唯一性与已有索引相同)",
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// parse_project - 反序列化 + 校验合一(常用入口,对齐 legacy parseProject)。
pub fn parse_project(value: serde_json::Value) -> Result<Project, ParseError> {
    let project: Project = serde_json::from_value(value)?;
    validate_project(&project).map_err(ParseError::Validate)?;
    Ok(project)
}

impl Project {
    /// from_json - 反序列化(JSON Value -> Project),纯结构层(不含业务校验)。
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proj_with_table(json: &str) -> Project {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn duplicate_code_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"a","name":"a","dataType":"VARCHAR","length":32},
                {"code":"A","prop":"a2","name":"a2","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("重复")));
    }

    #[test]
    fn duplicate_prop_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"x","name":"a","dataType":"VARCHAR","length":32},
                {"code":"B","prop":"x","name":"b","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.path.ends_with(".prop") && e.message.contains("重复")));
    }

    #[test]
    fn sql_reserved_code_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"VALUE","prop":"v","name":"v","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("SQL 保留字")));
    }

    #[test]
    fn java_keyword_prop_reported() {
        // code=CLASS -> prop=class(Java 关键字)
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"X","prop":"class","name":"x","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("Java 关键字")));
    }

    #[test]
    fn decimal_p_less_than_s_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"M","prop":"m","name":"m","dataType":"DECIMAL","precision":2,"scale":4}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("不能小于 scale")));
    }

    #[test]
    fn duplicate_index_name_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"a","name":"a","dataType":"VARCHAR","length":32}
            ],"indexes":[
                {"name":"IDX1","fields":[{"code":"A","direction":"ASC"}],"unique":false},
                {"name":"IDX1","fields":[{"code":"A","direction":"ASC"}],"unique":false}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("索引名重复")));
    }

    #[test]
    fn empty_index_name_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"a","name":"a","dataType":"VARCHAR","length":32}
            ],"indexes":[
                {"fields":[{"code":"A","direction":"ASC"}],"unique":false}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("索引名不能为空")));
    }

    #[test]
    fn duplicate_index_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"a","name":"a","dataType":"VARCHAR","length":32}
            ],"indexes":[
                {"fields":[{"code":"A","direction":"ASC"}],"unique":false},
                {"fields":[{"code":"A","direction":"ASC"}],"unique":false}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("索引重复")));
    }

    #[test]
    fn empty_length_precision_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"a","name":"a","dataType":"VARCHAR"},
                {"code":"M","prop":"m","name":"m","dataType":"DECIMAL","precision":10}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.path.ends_with(".length") && e.message.contains("不能为空")));
        assert!(errs.iter().any(|e| e.path.ends_with(".scale") && e.message.contains("不能为空")));
    }

    #[test]
    fn empty_prop_name_reported() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"A","prop":"","name":"","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        let errs = validate_project(&p).unwrap_err();
        assert!(errs.iter().any(|e| e.path.ends_with(".prop") && e.message.contains("不能为空")));
        assert!(errs.iter().any(|e| e.path.ends_with(".name") && e.message.contains("不能为空")));
    }

    #[test]
    fn valid_project_no_errors() {
        let p = proj_with_table(
            r#"{"version":"1","basePackage":"x","bizTypes":[],"groups":[],"tables":[{"code":"T","name":"T","group":"","fields":[
                {"code":"USER_NAME","prop":"userName","name":"用户名","dataType":"VARCHAR","length":32}
            ]}]}"#,
        );
        assert!(validate_project(&p).is_ok());
    }
}
