//! §3.4 业务类型(BizType)定义。

use crate::schema::data_type::DataType;
use serde::{Deserialize, Serialize};

/// §3.4 支持的数据类型配置项（supportedDataTypes 元素）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedDataType {
    #[serde(rename = "dataType")]
    pub data_type: DataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultLength")]
    pub default_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultPrecision")]
    pub default_precision: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultScale")]
    pub default_scale: Option<u32>,
}

/// §3.4 业务类型参数配置（前端表单生成用）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BizTypeDataField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: BizTypeDataFieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// 参数默认值（跟随 field_type:string/number）。选 bizType 时用于初始化 bizTypeData。
    #[serde(rename = "default", default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BizTypeDataFieldType {
    String,
    Number,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BizTypeData {
    pub fields: Vec<BizTypeDataField>,
}

/// §3.4 业务类型 BizTypeDefine（内置 + 自定义共用此结构）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BizTypeDefine {
    #[serde(rename = "bizType")]
    pub biz_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "supportedDataTypes")]
    pub supported_data_types: Vec<SupportedDataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizTypeData")]
    pub biz_type_data: Option<BizTypeData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_without_default_backward_compat() {
        // 旧 schema:无 default 字段
        let json = r#"{"name":"format","type":"string","required":true}"#;
        let f: BizTypeDataField = serde_json::from_str(json).unwrap();
        assert_eq!(f.name, "format");
        assert!(f.default_value.is_none());
    }

    #[test]
    fn test_field_with_string_default() {
        let json = r#"{"name":"format","type":"string","default":"YYYYMMDD"}"#;
        let f: BizTypeDataField = serde_json::from_str(json).unwrap();
        assert_eq!(
            f.default_value,
            Some(serde_json::Value::String("YYYYMMDD".into()))
        );
    }

    #[test]
    fn test_field_with_number_default() {
        let json = r#"{"name":"scale","type":"number","default":2}"#;
        let f: BizTypeDataField = serde_json::from_str(json).unwrap();
        assert_eq!(f.default_value, Some(serde_json::Value::Number(2.into())));
    }

    #[test]
    fn test_skip_serializing_none_default() {
        let f = BizTypeDataField {
            name: "x".into(),
            field_type: BizTypeDataFieldType::String,
            description: None,
            required: None,
            default_value: None,
        };
        let s = serde_json::to_string(&f).unwrap();
        assert!(!s.contains("default"));
    }
}
