# generators-ddl 实现计划

主会话直接实现(首个 generator,建立范式)。逐文件实现,每步可独立编译验证。

## 实施顺序

1. [ ] `generators/mod.rs` - 模块声明 + pub use ddl
2. [ ] `generators/ddl/mod.rs` - Dialect enum + generate_ddl 入口 + DdlOptions
3. [ ] `generators/ddl/types.rs` - map_type 类型映射(9×7=63 分支)
4. [ ] `generators/ddl/table.rs` - generate_table / field_definition / COMMENT 逻辑
5. [ ] `generators/ddl/index.rs` - generate_index / auto_index_name
6. [ ] `tests/generators_ddl.rs` - 集成测试 + fixtures
7. [ ] `cargo test -p aqua-core` 验证

## 每步验证命令

```bash
cargo check -p aqua-core              # 编译检查
cargo test -p aqua-core               # 完成后
cargo clippy -p aqua-core -- -D warnings
```

## 依赖关系

- generators/ddl 依赖 schema 模块(Project/Table/Field/DataType)
- 测试依赖 schema fixtures(valid-full.json)

## 完成标准

- [ ] 7 方言 DDL 可生成
- [ ] CREATE TABLE + PRIMARY KEY + COMMENT
- [ ] CREATE INDEX (普通/UNIQUE,自动命名)
- [ ] 表/分组过滤
- [ ] 测试覆盖主流程(MySQL/PG)
- [ ] clippy / fmt 通过

## 回滚点

- 类型映射(步骤 3): 单独可测,出错回退
- COMMENT 方言分支(步骤 4): 复杂逻辑,独立验证
- 测试(步骤 6): fixtures 不通过回看映射规则

## Phase 3.3 spec 回填

完成后回填 `.trellis/spec/aqua-core/backend/`:
- 新增 `generators-guidelines.md`: generator 规范(纯函数/无 I/O/类型映射模式)
- 更新 `directory-structure.md`: generators 模块组织
