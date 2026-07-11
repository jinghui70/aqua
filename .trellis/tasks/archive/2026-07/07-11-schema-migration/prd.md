# schema 模块移植至 Rust

## 背景
aqua v2 移植起点(architecture.md §8)。JSON SSOT 的核心类型定义,后续 generators / dataset / import / diff 全部依赖。
- 逻辑蓝本: `~/work/aqua-legacy/packages/core/src/schema/`(TS + zod,~340 行)
- 业务定义: `docs/design.md` §3 数据模型
- 现状: `crates/aqua-core/src/schema/mod.rs` 仅有占位注释

## 目标
将 schema 数据模型从 legacy TS+zod 移植为 Rust + serde,作为 aqua-core 第一个可测模块。

**包含**:
- 类型定义: DataType / Field / AutoGenerate / FieldEnum / Table / Index / BizTypeDefine / SupportedDataType / BizTypeData / EnumDefine / InlineEnum / EnumValue / EnumColor / GroupDefine / Project
- 反序列化: serde JSON <-> Rust 结构(双向)
- 业务校验: enum 仅 VARCHAR / hasCode 一致性 / values 非空 / 必填
- 对外 API: `parse_project` / `validate_project` / `Project::from_json`(对齐 legacy `parseProject`/`validateProject`)
- 测试: 移植 legacy 6 个用例 + 4 个 fixtures + serde 往返测试

## 范围(不含)
- prop 从 code 派生(蛇形->驼峰):UI/编辑层逻辑,legacy schema 无此函数,后续 UI 任务做
- 内置 bizType 清单:产品配置文件,待定(design.md §11)
- DDL/代码生成:generators 模块
- 任何 I/O:纯逻辑,I/O 在 src-tauri 层(lib.rs 已声明)

## 约束
- 纯逻辑核心,无 I/O 耦合
- 类型加 serde derive 支持 JSON 序列化
- Rust 无 zod"类型+校验同源"机制 -> serde 类型层 + 独立 validate 函数(见 design.md)

## 验收标准
- [ ] `crates/aqua-core/src/schema/` 下 7 个类型文件 + validate.rs + mod.rs
- [ ] `cargo test -p aqua-core` 全绿(含移植 6 用例 + 往返测试)
- [ ] `cargo clippy -p aqua-core -- -D warnings` 无 warning
- [ ] `cargo fmt --check` 通过
- [ ] serde 往返等价(JSON -> Rust -> JSON)
- [ ] 校验覆盖: enum 仅 VARCHAR / hasCode 一致 / values 非空 / 必填
