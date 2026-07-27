//! §3.1 逻辑类型（DataType）- 10 种。
//! 不含 BOOLEAN / JSON（设计文档 §3.1 说明）。
//! DOUBLE 为 IEEE 754 双精度浮点(非金额物理量);金额必须用 DECIMAL。

use serde::{Deserialize, Serialize};

/// 10 种逻辑数据类型,序列化为大写(对齐 legacy JSON)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataType {
    Varchar,
    Clob,
    Tinyint,
    Int,
    Long,
    Decimal,
    /// 双精度浮点(IEEE 754)。非金额物理量专用,不允许 precision/scale。
    /// 金额/精确小数必须用 DECIMAL。
    Double,
    Date,
    Datetime,
    Blob,
}
