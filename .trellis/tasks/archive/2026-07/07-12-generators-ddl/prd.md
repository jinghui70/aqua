# generators 模块移植: DDL 生成器

## 背景

aqua v2 移植路线第二步(schema 已完成)。DDL 生成器是验证 schema 模型正确性的最快路径,从 Project JSON 生成 CREATE TABLE/INDEX DDL。

- 逻辑蓝本: `~/work/aqua-legacy/packages/core/src/generators/ddl/`
- 业务规范: `docs/design.md` §4 DDL 生成规则
- 现状: `crates/aqua-core/src/` 无 generators 模块

## 目标

实现 DDL 生成器,支持 7 种方言,从 Project 生成完整 CREATE TABLE/INDEX DDL。

**包含**:
- 类型映射: 9 种逻辑类型 → 各方言物理类型
- CREATE TABLE 生成: 字段定义/PRIMARY KEY/COMMENT
- CREATE INDEX 生成: 普通索引/UNIQUE 索引
- 7 方言支持: MySQL/PostgreSQL/Oracle/DM/KingBase/GBase/H2
- 默认值处理: defaultValue 转义
- 注释生成: 表注释/字段注释

## 范围(不含)

- ALTER DDL(diff 任务做)
- 数据库执行(aqua 只生成文本,不执行)
- autoGenerate(应用层逻辑,不进 DDL)
- bizType(不影响 DDL,元信息)

## 验收标准

**实现**:
- [ ] `crates/aqua-core/src/generators/mod.rs` 模块声明
- [ ] `crates/aqua-core/src/generators/ddl/mod.rs` DDL 生成器入口
- [ ] `crates/aqua-core/src/generators/ddl/types.rs` 类型映射(9 逻辑类型 → 7 方言)
- [ ] `crates/aqua-core/src/generators/ddl/mysql.rs` MySQL 方言
- [ ] `crates/aqua-core/src/generators/ddl/postgres.rs` PostgreSQL 方言
- [ ] `crates/aqua-core/src/generators/ddl/oracle.rs` Oracle 方言
- [ ] `crates/aqua-core/src/generators/ddl/信创.rs` DM/KingBase/GBase(三种信创库)
- [ ] `crates/aqua-core/src/generators/ddl/h2.rs` H2 方言

**测试**:
- [ ] `tests/generators/ddl.rs` 集成测试
- [ ] `tests/fixtures/ddl/` DDL 输出样本(各方言)
- [ ] valid-full.json → DDL 往返验证(DDL → 手动建库 → 导入 → JSON 对比)

**质量**:
- [ ] `cargo test -p aqua-core` 全绿
- [ ] `cargo clippy -p aqua-core -- -D warnings` 无 warning
- [ ] DDL 输出可直接在目标库执行(手动验证 MySQL/PG)

## 约束

- 纯逻辑,无 I/O(DDL 返回 String,不写文件)
- 类型映射遵循 design.md §4.1 规则
- 注释转义(SQL 注入防护)
- 索引名自动生成(无 name 时: `idx_<table>_<field1>_<field2>`)

## 参考

- legacy 实现: `~/work/aqua-legacy/packages/core/src/generators/ddl/`
- 类型映射表: design.md §4.1
- 测试用例: legacy `__tests__/generators/ddl.test.ts`
