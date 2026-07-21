//! §3.2 字段模型(Field)与自动生成配置(AutoGenerate)。

use crate::schema::data_type::DataType;
use crate::schema::enum_def::InlineEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// §3.2 autoGenerate - 应用层生成（对齐 @GeneratedValue），不进 DDL。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoGenerate {
    pub enabled: bool,
    pub strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    pub timing: GenerateTiming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenerateTiming {
    Insert,
    InsertUpdate,
}

/// §3.2 字段模型 Field（对齐 json-ui DataFieldSchema + 工具扩展）。
///
/// 字段声明顺序即 JSON 序列化顺序:code/prop/name 靠前(标识优先),
/// bizType/bizTypeData 靠后(业务扩展,阅读时不干扰主信息)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub code: String,
    pub prop: String,
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: DataType,

    // 类型属性
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,

    // 约束（对齐 json-ui）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isKey")]
    pub is_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "notNull")]
    pub not_null: Option<bool>,

    // 工具扩展
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,

    // 应用层生成
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "autoGenerate")]
    pub auto_generate: Option<AutoGenerate>,

    // Enum：字段内联枚举（无全局引用，统一 InlineEnum）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_ref: Option<InlineEnum>,

    // 业务类型（对齐 json-ui，靠后放置）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizType")]
    pub biz_type: Option<String>,
    /// 单 field 存值、多 field 存对象（§3.4），此处保留任意 JSON (对齐 legacy z.unknown())。
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bizTypeData")]
    pub biz_type_data: Option<Value>,

    // 文档（不进 DDL）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
