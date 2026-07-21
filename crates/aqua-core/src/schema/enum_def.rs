//! §3.5 内联枚举(InlineEnum) - 字段级枚举定义(无全局枚举)。

use serde::{Deserialize, Serialize};

/// §3.5 EnumColor 预置 13 色（写死代码，改需改代码）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnumColor {
    Success,
    Error,
    Warning,
    Info,
    Primary,
    Danger,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Grey,
}

/// §3.5 EnumValue - 枚举值项。
/// code: hasCode=true 时必填(由 validate 层校验)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValue {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<EnumColor>,
}

/// §3.5 InlineEnum - field.enum 内联枚举（无 code/package）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineEnum {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasCode")]
    pub has_code: Option<bool>,
    pub values: Vec<EnumValue>,
}
