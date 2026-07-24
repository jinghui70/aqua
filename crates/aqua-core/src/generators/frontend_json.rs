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
///
/// 字段声明顺序即序列化顺序:code/prop/name 靠前,bizType/bizTypeData 靠后。
/// 注意:序列化必须直接走 struct(见 generate_frontend_json),不能经 serde_json::Value 中转,
/// 否则 Value::Object 的 BTreeMap 会把键重排成字母序。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiField {
    pub code: String,
    pub prop: String,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: JsonUiDataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(rename = "isKey")]
    pub is_key: bool,
    #[serde(rename = "notNull")]
    pub not_null: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizType")]
    pub biz_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizTypeData")]
    pub biz_type_data: Option<serde_json::Value>,
}

/// json-ui DataModel(JsonModelSchema:type/code/name/fields)。
/// 对齐 json-ui `src/schema/json-file.ts` 的 JsonModelSchema(存为 *.model.json)。
#[derive(Debug, Clone, Serialize)]
pub struct JsonUiModel {
    /// 固定 "model",json-ui FileEngine 据此识别模型文件。
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub code: String,
    pub name: String,
    pub fields: Vec<JsonUiField>,
}

/// 前端 JSON 生成选项。
#[derive(Debug, Clone, Default)]
pub struct FrontendJsonOptions {
    /// 单表过滤(为空则取首表;json-ui model 是单表概念)
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

/// Table -> JsonUiModel 转换(JsonModelSchema,type 固定 "model")。
pub fn transform_table(table: &Table) -> JsonUiModel {
    JsonUiModel {
        type_: "model",
        code: table.code.clone(),
        name: table.name.clone(),
        fields: table.fields.iter().map(transform_field).collect(),
    }
}

/// 前端 JSON 生成入口,返回单表 json-ui JsonModelSchema 文本。
/// options.table 指定表(为空取首表);model 是单表概念,不再包裹 tables 数组。
pub fn generate_frontend_json(project: &Project, options: &FrontendJsonOptions) -> String {
    let table: &Table = if let Some(ref table_code) = options.table {
        project
            .tables
            .iter()
            .find(|t| t.code == *table_code)
            .unwrap_or_else(|| panic!("Table not found: {}", table_code))
    } else {
        project
            .tables
            .first()
            .expect("项目无表,无法生成 model")
    };

    // 直接序列化 struct 保持字段顺序;经 serde_json::Value 会被 BTreeMap 重排成字母序
    serde_json::to_string_pretty(&transform_table(table)).unwrap()
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

    #[test]
    fn test_field_order_code_prop_name_first_biztype_last() {
        // 序列化字段顺序: code/prop/name 靠前, bizType/bizTypeData 靠后
        // (防回归: 经 serde_json::Value 中转会被 BTreeMap 重排成字母序)
        let field = Field {
            code: "NAME".to_string(),
            prop: "name".to_string(),
            name: "名字".to_string(),
            data_type: DataType::Varchar,
            length: Some(8),
            precision: None,
            scale: None,
            biz_type: Some("Date".to_string()),
            biz_type_data: Some(serde_json::json!("YYYYMMDD")),
            is_key: Some(false),
            not_null: Some(true),
            auto_generate: None,
            default_value: None,
            enum_ref: None,
            comment: None,
        };
        let project = Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            tables: vec![Table {
                code: "T".to_string(),
                name: "表".to_string(),
                group: "g".to_string(),
                fields: vec![field],
                indexes: None,
                comment: None,
            }],
            biz_types: vec![],
        auto_gen_strategies: vec![],
            groups: vec![],
        };
        let json = generate_frontend_json(&project, &FrontendJsonOptions::default());

        // JsonModelSchema:顶层 type=model + code/name(单表,无 tables 包裹)
        assert!(json.contains("\"type\": \"model\""), "顶层应有 type:model:\n{}", json);
        assert!(!json.contains("\"tables\""), "不应再有 tables 包裹:\n{}", json);

        // 只看 field 对象片段(table 也有 name 字段,避免 find 匹配到 table.name)
        let field_json = &json[json.find("\"fields\"").unwrap()..];
        let pos = |k: &str| field_json.find(k).unwrap_or(usize::MAX);
        // code < prop < name < dataType
        assert!(pos("\"code\"") < pos("\"prop\""), "code 应在 prop 前:\n{}", json);
        assert!(pos("\"prop\"") < pos("\"name\""), "prop 应在 name 前:\n{}", json);
        assert!(pos("\"name\"") < pos("\"dataType\""), "name 应在 dataType 前:\n{}", json);
        // bizType/bizTypeData 靠后(在 notNull 之后)
        assert!(pos("\"notNull\"") < pos("\"bizType\""), "bizType 应靠后:\n{}", json);
    }
}
