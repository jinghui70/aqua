//! 枚举定义索引 - 供 Java / frontend_json 生成器共享。
//!
//! 定义方 = 字段有 enum 且 enum.ref=None;引用方 = enum.ref=Some(不在此索引)。
//! 键 = (table.code, field.prop),与 enum.ref 的锚点一致。

use crate::schema::{Field, InlineEnum, Project};
use std::collections::HashMap;

/// 枚举定义(定义方字段)。类名由生成器按 naming::prop_to_pascal 派生(此处不存,避免 schema→generators 依赖)。
#[derive(Debug, Clone)]
pub struct EnumDefine {
    /// 所属表 code
    pub table_code: String,
    /// 定义字段 prop
    pub field_prop: String,
    /// 定义字段本身(含 enum_ref)
    pub field: Field,
    /// 枚举定义
    pub r#enum: InlineEnum,
}

impl Project {
    /// 收集全项目所有枚举定义(仅定义方,即 enum.ref=None)。
    pub fn enum_defs(&self) -> HashMap<(String, String), EnumDefine> {
        let mut defs: HashMap<(String, String), EnumDefine> = HashMap::new();
        for table in self.tables.iter() {
            for field in table.fields.iter() {
                let Some(e) = &field.enum_ref else { continue };
                if e.r#ref.is_some() {
                    continue; // 引用方不入索引
                }
                defs.insert(
                    (table.code.clone(), field.prop.clone()),
                    EnumDefine {
                        table_code: table.code.clone(),
                        field_prop: field.prop.clone(),
                        field: field.clone(),
                        r#enum: e.clone(),
                    },
                );
            }
        }
        defs
    }
}
