//! 数据模型 - JSON SSOT 的核心类型定义。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/schema/`(TS),对齐 `docs/design.md` §3 数据模型。
//! 类型加 `serde` derive 以支持 JSON 序列化。

mod biz_type;
mod data_type;
mod enum_def;
mod field;
mod project;
mod table;
mod validate;

// Re-export 公共类型
pub use biz_type::{BizTypeData, BizTypeDataField, BizTypeDefine, SupportedDataType};
pub use data_type::DataType;
pub use enum_def::{EnumColor, EnumDefine, EnumValue, InlineEnum};
pub use field::{AutoGenerate, Field, FieldEnum, GenerateTiming};
pub use project::{GroupDefine, Project};
pub use table::{Direction, Index, IndexField, Table};
pub use validate::{parse_project, validate_project, ParseError, ValidationError};
