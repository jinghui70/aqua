//! generators 模块 - DDL/Java/JSON/StrConst 生成器。
//!
//! 纯逻辑,无 I/O。输入已校验的 Project,输出代码/DDL 文本。

pub mod ddl;
pub mod java;

// 后续其他 generators:
// pub mod frontend_json;
// pub mod strconst;
