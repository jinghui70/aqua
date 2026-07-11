# schema 移植执行计划

主会话直接实现(不 dispatch sub-agent,首个代码建立范式)。逐文件移植,每步可独立 `cargo check`。

## 顺序

1. [ ] `data_type.rs` — DataType enum + serde UPPERCASE
2. [ ] `enum_def.rs` — EnumColor / EnumValue / InlineEnum / EnumDefine
3. [ ] `biz_type.rs` — SupportedDataType / BizTypeDataField / BizTypeData / BizTypeDefine
4. [ ] `field.rs` — AutoGenerate / FieldEnum(untagged) / Field
5. [ ] `table.rs` — Index / Table
6. [ ] `project.rs` — GroupDefine / Project
7. [ ] `validate.rs` — ValidationError / ParseError / validate_project / parse_project / from_json
8. [ ] `mod.rs` — re-export,删占位注释
9. [ ] `tests/fixtures/` — 移植 4 个 fixture JSON(从 legacy)
10. [ ] `tests/schema.rs` — 移植 6 用例 + serde 往返测试
11. [ ] `cargo test` / `clippy -D warnings` / `fmt --check` 全绿

## 依赖说明
data_type 无依赖;enum_def 独立;biz_type 依赖 data_type;field 依赖 data_type + enum_def;table 依赖 field;project 依赖 table/biz_type/enum_def;validate 依赖全部。按序 1->7 编译链不断。

## 验证命令
```bash
cargo check -p aqua-core              # 每步后
cargo test -p aqua-core               # 完成后
cargo clippy -p aqua-core -- -D warnings
cargo fmt -p aqua-core -- --check
```

## 回滚点
- 类型文件 1-6:每完成一个 `cargo check` 通过即检查点
- validate.rs(步骤 7)是最复杂一步,单独验证校验规则覆盖
- 测试(步骤 9-10)失败 -> 回看 fixture 与 legacy 是否一致

## 完成后(Phase 3.3 spec 回填)
把 schema 模块涌现的真实约定回填 `.trellis/spec/aqua-core/backend/`:
- `directory-structure.md`: 模块逐文件拆分、文件名对齐 legacy、enum 关键字避让
- `error-handling.md`: thiserror + Result、ValidationError 带 path、校验收集不短路
- `quality-guidelines.md`: serde derive 必备、clippy -D warnings、测试 + fixtures 风格
顺带推进 `00-bootstrap-guidelines` 的 aqua-core 部分。
