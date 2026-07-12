//! 前端 JSON 生成器 - 生成 json-ui 兼容格式(服务于外部 json-ui 项目)。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/frontend-json/`。
//! 规则见 `docs/design.md` §4.2.2。

use crate::schema::{DataType, Field, Project, Table};
use serde::{Deserialize, Serialize};

/// json-ui 粗粒度数据类型(4 种)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JsonUiDataType {
    Number,
    String,
    Date,
    Datetime,
}

/// 9 逻辑类型 -> json-ui 4 粗粒度类型。
pub fn map_data_type(dt: DataType) -> JsonUiDataType {
    match dt {
        DataType::Int | DataType::Long | DataType::Decimal | DataType::Tinyint => {
            JsonUiDataType::Number
        }
        DataType::Varchar | DataType::Clob | DataType::Blob => JsonUiDataType::String,
        DataType::Date => JsonUiDataType::Date,
        DataType::Datetime => JsonUiDataType::Datetime,
    }
}

/// json-ui Field(排除 precision/autoGenerate/comment)。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiField {
    pub prop: String,
    pub code: String,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: JsonUiDataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizType")]
    pub biz_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizTypeData")]
    pub biz_type_data: Option<serde_json::Value>,
    #[serde(rename = "isKey")]
    pub is_key: bool,
    #[serde(rename = "notNull")]
    pub not_null: bool,
}

/// json-ui Table。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiTable {
    pub code: String,
    pub name: String,
    pub fields: Vec<JsonUiField>,
}

/// 前端 JSON 生成选项。
#[derive(Debug, Clone, Default)]
pub struct FrontendJsonOptions {
    /// 单表过滤(为空则全部表)
    pub table: Option<String>,
}

/// Field -> JsonUiField 转换(排除 precision/autoGenerate/comment)。
pub fn transform_field(field: &Field) -> JsonUiField {
    JsonUiField {
        prop: field.prop.clone(),
        code: field.code.clone(),
        name: field.name.clone(),
        data_type: map_data_type(field.data_type),
        length: field.length,
        scale: field.scale,
        biz_type: field.biz_type.clone(),
        biz_type_data: field.biz_type_data.clone(),
        is_key: field.is_key.unwrap_or(false),
        not_null: field.not_null.unwrap_or(false),
    }
}

/// Table -> JsonUiTable 转换。
pub fn transform_table(table: &Table) -> JsonUiTable {
    JsonUiTable {
        code: table.code.clone(),
        name: table.name.clone(),
        fields: table.fields.iter().map(transform_field).collect(),
    }
}

/// 前端 JSON 生成入口,返回 json-ui 兼容 JSON 文本。
pub fn generate_frontend_json(project: &Project, options: &FrontendJsonOptions) -> String {
    let tables: Vec<&Table> = if let Some(ref table_code) = options.table {
        vec![project
            .tables
            .iter()
            .find(|t| t.code == *table_code)
            .unwrap_or_else(|| panic!("Table not found: {}", table_code))]
    } else {
        project.tables.iter().collect()
    };

    let transformed: Vec<JsonUiTable> = tables.iter().map(|t| transform_table(t)).collect();
    serde_json::to_string_pretty(&serde_json::json!({ "tables": transformed })).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_data_type() {
        assert_eq!(map_data_type(DataType::Int), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Long), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Decimal), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Tinyint), JsonUiDataType::Number);
        assert_eq!(map_data_type(DataType::Varchar), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Clob), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Blob), JsonUiDataType::String);
        assert_eq!(map_data_type(DataType::Date), JsonUiDataType::Date);
        assert_eq!(map_data_type(DataType::Datetime), JsonUiDataType::Datetime);
    }

    #[test]
    fn test_transform_field_excludes_precision() {
        let field = Field {
            prop: "amount".to_string(),
            code: "AMOUNT".to_string(),
            name: "金额".to_string(),
            data_type: DataType::Decimal,
            length: None,
            precision: Some(12),
            scale: Some(2),
            biz_type: None,
            biz_type_data: None,
            is_key: Some(false),
            not_null: Some(true),
            auto_generate: None,
            default_value: None,
            enum_ref: None,
            comment: Some("备注".to_string()),
        };

        let json = transform_field(&field);
        let serialized = serde_json::to_string(&json).unwrap();

        // 包含核心字段
        assert!(serialized.contains("\"prop\":\"amount\""));
        assert!(serialized.contains("\"code\":\"AMOUNT\""));
        assert!(serialized.contains("\"dataType\":\"NUMBER\""));
        assert!(serialized.contains("\"scale\":2"));
        assert!(serialized.contains("\"notNull\":true"));

        // 排除 precision/autoGenerate/comment
        assert!(!serialized.contains("precision"));
        assert!(!serialized.contains("autoGenerate"));
        assert!(!serialized.contains("comment"));
        assert!(!serialized.contains("备注"));
    }
}
