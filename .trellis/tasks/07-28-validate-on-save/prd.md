# 保存时合法性校验

## Goal

项目保存时自动清理空字段/无效索引，对剩余内容做合法性校验，错误列表（按表分组）confirm 提示"仍保存"/"取消去修"。出口把关，编辑过程不阻断；打开项目时不因校验问题阻断。

## Background

- 后端 `validate_project`（`crates/aqua-core/src/schema/validate.rs`）原仅校验数据类型属性 + enum 规则。已扩展为覆盖 code/prop 重复与关键字、非空、p<s、索引结构等。
- 前端 `useTauri.projectValidate` 已接上（返回 `ValidationError[]`）。`store.saveProject` 落盘前清理 + 校验 + confirm。
- `project_open` 改为只反序列化（`Project::from_json`），校验问题不阻止打开，打开后 alert 提示。

## Requirements

- R1 保存时清理（前端 store，落盘前，改 currentProject 使 UI 同步）：
  - 字段 `code` 空 -> 删字段。
  - 索引：删 fields 中 `code` 空/不在表 fields 里的 field；fields 按 `code` 去重（保留首次）；fields 空了删索引。
- R2 校验规则（后端 `validate_project` 扩展，复用 `ValidationError{path,message}`，不短路）：
  - code 表内重复、code 撞 SQL 保留字、prop 撞 Java 关键字、prop 表内重复。
  - prop/name 不能为空；VARCHAR length 不能为空；DECIMAL precision/scale 不能为空。
  - DECIMAL precision < scale 报错。
  - 索引名不能为空（不自动生成）、索引名表内重复、索引重复（fields 序列 + unique 完全相同）。
- R3 关键字：SQL 保留字（ANSI + MySQL/PG/Oracle/H2 并集）+ Java 关键字（JLS），后端 `keywords.rs` const 集合。code 查 SQL（大小写不敏感），prop 查 Java（精确）。
- R4 接入 `saveProject`：落盘前 cleanup -> `projectValidate` -> 有错 `ElMessageBox.confirm`。
- R5 confirm：错误列表按表分组（表 code 标题 + 错误数），条目 path 去表 code 前缀 + message；按钮"仍保存"/"取消去修"。取消不落盘。无错直接落盘。
- R6 `saveProject` 返回 `boolean`；调用方适配（`confirmIfDirty` 取消则不关闭、`NewProjectDialog` 取消不进入项目、`useMenuActions` 取消不提示已保存）。
- R7 清理静默（不单独提示清理数）。
- R8 IndexTab 索引字段 el-select 选项过滤当前索引已选 code（预防重复选择）。
- R9 索引名不自动生成（必填）；新增索引默认填 `IDX_<表>_<序号>`，序号扫描已有 `IDX_<表>_N` 取最大 +1。
- R10 `project_open` 只反序列化（结构坏才打不开）；打开后 `projectValidate`，有错 `ElMessageBox.alert` 提示（不阻止打开）。
- R11 删后端 `auto_index_name`（`generate_index` 空名 `unwrap_or_default`，校验保证不触发）。
- R12 path 格式：字段 `{table.code}.{field.code}.属性`，索引 `{table.code}.[索引名]`；显示去表 code 前缀（分组标题已有）。message 简洁（不能为空 / 重复: 值 / 索引名重复 / 索引重复(...)）。

## Key Decisions

- D1 清理前端 store / 校验后端 validate_project / confirm 前端。
- D2 时机=保存时（非实时非生成时）。
- D3 confirm 不强制阻塞（仍保存/取消去修）。
- D4 关键字跨库并集（偏严不偏松）。
- D5 saveProject 返回 boolean，confirmIfDirty 取消则不关闭。
- D6 索引名必填不自动生成（用户填，默认 IDX_表_序号）；后端 auto_index_name 删除。
- D7 打开不阻断校验（只反序列化 + alert 提示），允许带问题保存后再次打开。
- D8 显示按表分组 + path 去表 code 前缀 + message 去重复前缀。

## Acceptance Criteria

- [ ] 保存：code 空字段删；索引清空 field/不存在 field/重复 field 去重/空索引删；UI 反映。
- [ ] code/prop 表内重复报错；code 撞 SQL 保留字、prop 撞 Java 关键字报错。
- [ ] prop/name/VARCHAR length/DECIMAL precision·scale 空报"不能为空"；DECIMAL p<s 报错。
- [ ] 索引名空报错；索引名重复报"索引名重复"；索引 fields+unique 完全相同报"索引重复"。
- [ ] 有错弹 confirm（按表分组，path 去表 code）；仍保存落盘 / 取消去修不落盘。
- [ ] 无错直接落盘。
- [ ] confirmIfDirty 自动保存校验取消 -> 不关闭项目。
- [ ] 新增索引默认名 `IDX_表_1`、再增 `IDX_表_2`；IndexTab 已选 code 不出现在其他字段选项。
- [ ] 带问题保存后关闭 -> 重新打开能打开 + alert 提示问题。
- [ ] 既有数据类型属性/enum 校验不回归。

## Out of Scope

- 实时行内校验（编辑时不校验）。
- 生成 DDL/Java 时校验（保存时已校验）。
- 关键字按数据源类型精确区分（用跨库并集）。
- 字段编辑 UI（已完成于 07-28-field-cell-edit-dialog）。
