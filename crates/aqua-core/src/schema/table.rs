//! §3.3 表模型(Table)与索引(Index)。

use crate::schema::field::Field;
use serde::{Deserialize, Serialize};

/// §3.3 索引 Index。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub fields: Vec<String>,
    pub unique: bool,
}

/// §3.3 表模型 Table。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub code: String,
    pub name: String,
    pub group: String,
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexes: Option<Vec<Index>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
