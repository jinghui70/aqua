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

/// §3.5 InlineEnumRef - 枚举引用(跨表共享)。
/// 引用方字段的 enum.ref 指向定义方(表 code + 字段 prop),定义方 enum.ref=None。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineEnumRef {
    /// 定义方表 code
    pub code: String,
    /// 定义方字段 prop(一字段一枚举,prop 唯一稳定,锚定到字段)
    pub prop: String,
}

/// §3.5 InlineEnum - field.enum 内联枚举(无 code/package)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineEnum {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasCode")]
    pub has_code: Option<bool>,
    /// 显式枚举类名,缺省按定义字段 prop 派生 PascalCase。
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "className")]
    pub class_name: Option<String>,
    /// Some=引用型:指向定义方,不携带 values 副本,生成时按 ref 解析、不产代码。
    /// None=定义型(枚举在本字段定义)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<InlineEnumRef>,
    #[serde(default)]
    pub values: Vec<EnumValue>,
}
