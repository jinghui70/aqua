//! 业务校验层 - 独立于 serde 的语义规则校验。

use crate::schema::data_type::DataType;
use crate::schema::project::Project;
use serde::{Deserialize, Serialize};
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
/// 校验规则(§3 数据模型):
/// - enum 只支持 VARCHAR (field.rs 规则)
/// - hasCode=true 时每个 value 必须有 code (enum_def.rs 规则)
/// - values 非空 (已由 serde 保证: Vec 反序列化空数组成功,由此函数校验非空)
/// - 必填字段非空 (已由 serde 类型层保证: 缺失必填字段无法反序列化)
pub fn validate_project(project: &Project) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 校验全局枚举
    for (enum_idx, enum_def) in project.enums.iter().enumerate() {
        // values 非空
        if enum_def.values.is_empty() {
            errors.push(ValidationError::new(
                format!("enums[{}].values", enum_idx),
                "values 数组不能为空",
            ));
        }

        // hasCode=true 时每个 value 必须有 code
        if enum_def.has_code.unwrap_or(false) {
            for (value_idx, value) in enum_def.values.iter().enumerate() {
                if value.code.is_none() || value.code.as_ref().unwrap().is_empty() {
                    errors.push(ValidationError::new(
                        format!("enums[{}].values[{}].code", enum_idx, value_idx),
                        "hasCode=true 时每个 value 必须有 code",
                    ));
                }
            }
        }
    }

    // 校验表
    for (table_idx, table) in project.tables.iter().enumerate() {
        // 校验字段
        for (field_idx, field) in table.fields.iter().enumerate() {
            // 规则: enum 只支持 VARCHAR
            if field.enum_ref.is_some() && field.data_type != DataType::Varchar {
                errors.push(ValidationError::new(
                    format!("tables[{}].fields[{}].enum", table_idx, field_idx),
                    format!("enum 只支持 VARCHAR，当前 dataType={:?}", field.data_type),
                ));
            }

            // 校验内联枚举
            if let Some(crate::schema::field::FieldEnum::Inline(inline_enum)) = &field.enum_ref {
                // values 非空
                if inline_enum.values.is_empty() {
                    errors.push(ValidationError::new(
                        format!("tables[{}].fields[{}].enum.values", table_idx, field_idx),
                        "values 数组不能为空",
                    ));
                }

                // hasCode=true 时每个 value 必须有 code
                if inline_enum.has_code.unwrap_or(false) {
                    for (value_idx, value) in inline_enum.values.iter().enumerate() {
                        if value.code.is_none() || value.code.as_ref().unwrap().is_empty() {
                            errors.push(ValidationError::new(
                                format!(
                                    "tables[{}].fields[{}].enum.values[{}].code",
                                    table_idx, field_idx, value_idx
                                ),
                                "hasCode=true 时每个 value 必须有 code",
                            ));
                        }
                    }
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
