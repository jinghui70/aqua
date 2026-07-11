//! 数据模型 - JSON SSOT 的核心类型定义。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/schema/`(TS),对齐 `docs/design.md` §3 数据模型。
//! 类型加 `serde` derive 以支持 JSON 序列化。
//!
//! TODO(Trellis 任务): 依次移植 project / table / field / bizType / enum / dataType,
//! 含校验逻辑(validate)与默认值推导。

// 移植起点: 从 legacy 的 schema/*.ts 逐个对应为 Rust struct + serde。
// 参考: ~/work/aqua-legacy/packages/core/src/schema/
