//! §3.2 自动生成策略定义(AutoGenStrategyDefine)。
//!
//! 全局定义(内置 + 自定义),字段 autoGenerate.strategy 引用 code。
//! 内置策略(default/now)在前端硬编码,自定义策略存 Project.autoGenStrategies。

use serde::{Deserialize, Serialize};

/// §3.2 自动生成策略定义(类似 BizTypeDefine,但更简单:仅 code/name/paramDesc)。
///
/// - code: 策略标识(如 "default" 雪花, "now" 当前时间)
/// - name: 中文名(如 "雪花id", "当前时间")
/// - paramDesc: 参数说明(有参数时作字段编辑 placeholder;无参数 None)
///
/// 策略最多 1 个字符串参数。有参数 -> paramDesc Some;无参数 -> paramDesc None。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoGenStrategyDefine {
    pub code: String,
    pub name: String,
    /// 参数说明(有参数时作 placeholder)。None = 无参数。
    #[serde(rename = "paramDesc", default, skip_serializing_if = "Option::is_none")]
    pub param_desc: Option<String>,
}
