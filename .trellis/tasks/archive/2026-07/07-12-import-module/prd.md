# 导入模块: 从数据库导入 schema

## 背景

aqua v2 P1 核心功能。导入模块是"从存量数据库反解为 aqua schema"的核心逻辑,利用 Driver trait 实现。

- 依赖: Driver trait 已定义,MySQL 驱动已实现
- 设计规范: `docs/architecture.md` §4 Driver trait
- 现状: Driver trait 可获取表/列/索引元数据,导入逻辑待实现

## 目标

实现导入模块,调用 Driver trait 反解数据库元数据,生成 aqua Project JSON。

**包含**:
- import_from_db 函数: DbConfig → Project
- 表反解: list_tables → Table
- 列反解: get_columns → Field
- 索引反解: list_indexes → Index
- 默认值处理: basePackage/group

## 范围(不含)

- 枚举识别(复杂,后续优化)
- 业务类型识别(需人工配置)
- 表分组智能推断(暂用默认分组)

## 验收标准

**实现**:
- [ ] `import/mod.rs` 模块入口
- [ ] `import/from_db.rs` import_from_db 实现
- [ ] 默认值: basePackage="com.example", group="default"
- [ ] Field.prop 自动生成(snake_to_camel)

**测试**:
- [ ] 单元测试: 元数据 → Field 转换
- [ ] 集成测试: Mock driver 导入(可选)

**质量**:
- [ ] `cargo test -p aqua-core` 通过
- [ ] `cargo clippy -- -D warnings` 无 warning

## 约束

- 纯逻辑,无 I/O(返回 Project)
- 错误统一为 DriverError
- 默认值合理(可后续编辑)

## 参考

- Driver trait: `crates/aqua-core/src/driver/trait_def.rs`
- schema 模型: `crates/aqua-core/src/schema/`
