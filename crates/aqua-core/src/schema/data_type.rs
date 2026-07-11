//! §3.1 逻辑类型（DataType）- 9 种。
//! 不含 BOOLEAN / JSON / DOUBLE（设计文档 §3.1 说明）。

use serde::{Deserialize, Serialize};

/// 9 种逻辑数据类型,序列化为大写(对齐 legacy JSON)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataType {
    Varchar,
    Clob,
    Tinyint,
    Int,
    Long,
    Decimal,
    Date,
    Datetime,
    Blob,
}
