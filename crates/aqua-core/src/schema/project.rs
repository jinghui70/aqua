//! §3.6 项目模型(Project)与分组(GroupDefine)。

use crate::schema::biz_type::BizTypeDefine;
use crate::schema::table::Table;
use serde::{Deserialize, Serialize};

/// §3.6 分组 GroupDefine（显式定义，数组顺序即排序）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupDefine {
    pub code: String,
    pub name: String,
}

/// §3.6 项目 Project（schema.json 顶层）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    /// 项目中文名(可选,旧 schema 兼容)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "basePackage")]
    pub base_package: String,
    #[serde(rename = "bizTypes")]
    pub biz_types: Vec<BizTypeDefine>,
    pub groups: Vec<GroupDefine>,
    pub tables: Vec<Table>,
}
