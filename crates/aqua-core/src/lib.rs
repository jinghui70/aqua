//! aqua-core: JSON-SSOT 数据库结构管理工具核心库。
//!
//! 纯逻辑核心(可独立测试,无 I/O 耦合)。I/O(spawn connector / 文件读写)在 `src-tauri` 层。
//! 技术架构见 `docs/architecture.md`,业务设计见 `docs/design.md`。
//!
//! ## 模块规划(移植起点: schema)
//! - `schema`: 数据模型(Project/Table/Field/Index/BizType/Enum/DataType) - 首先移植
//! - `generators`: DDL(7 方言)/Java 实体/前端 JSON/StrConst 生成
//! - `dataset`: JSON/SQLite 双格式数据集载体
//! - `import`: 连库导入(依赖 driver)
//! - `driver`: `trait Driver` + native(MySQL/PG)/Jdbc(Java connector) 两实现
//! - `diff`: JSON vs JSON diff + ALTER 生成(新功能)

pub mod generators;
pub mod schema;
// pub mod dataset;
// pub mod import;
// pub mod driver;
// pub mod diff;
