# 表编辑页改进

## Goal
完善表编辑页:分组/表 code 可编辑(唯一校验,页签不受影响)、字段弹窗加宽、备注后置、索引规范化(字段排序+方向)。让 code 成为可改的展示属性,页签用稳定 id 标识。

## Background
- 分组 code 编辑已实现(`updateGroup` `project.ts:188`,级联表 group + 唯一校验),纳入验收。
- 表 code 当前不可编辑(对话框 disabled `GroupTreeAside.vue:392`)。页签 key/path/title 用 `table.code` 生成(`project.ts:169-175`),改 code 会使旧页签指向"表不存在" + keep-alive 残留。
- 字段弹窗 `FieldDetailDialog` 600px;备注在基本区末位(后还有自动生成/业务类型区)。
- 索引 `Index.fields: string[]`(`schema.ts:56` / `table.rs:11`),无字段方向;DDL 无 ASC/DESC(`ddl/index.rs:14-16`);索引名自动生成后端已实现(`auto_index_name` `ddl/index.rs:30`),前端 placeholder 提示但未展示。
- 字段拖拽排序已实现(`FieldsTab.vue:30-52`)。

## Requirements

### R1 表 code 编辑(页签用 id 解耦)
- 表 code 可编辑(对话框去 disabled),唯一校验(与其他表 code 不重复)。
- 页签标识改用表运行时 id 而非 code:改 code 零影响页签与路由。
- id 运行时生成(打开项目加载时、新建表时),保存项目时不持久化。
- 树 node-key 用表 id(`table:${t.id}`),改 code 时树节点不重建。分组节点保持 `group:${g.code}`。

### R2 分组 code 编辑(已实现,纳入验收)
- `updateGroup` 改 code 级联更新 `tables[].group` + 唯一校验。

### R3 字段弹窗加宽 + 备注后置
- `FieldDetailDialog` 宽 600px -> 900px,基本区表单多列布局(利用宽度)。
- 备注移到弹窗最末(自动生成/业务类型区之后)。

### R4 索引规范化(字段排序 + 方向)
- 索引字段可排序,每字段可选排序方向(ASC/DESC)。
- DDL 生成含方向。
- 索引名留空时前端展示后端自动生成的名。
- 旧项目索引数据兼容(默认 ASC)。

### R5 索引与字段联动
- 新建索引默认带一个空字段(少一次点击)。
- 字段删除时,从该表所有索引中移除引用该字段 code 的项。
- 字段 code 改名时,同步该表索引中引用旧 code 的项(inline 编辑与详情弹窗两个入口都覆盖)。

## Acceptance Criteria
- [ ] R2:分组 code 改动后,该分组下所有表 `group` 引用同步;重复 code 报错。
- [ ] R1:表 code 改动后,已打开页签不失效、显示新 code;重复 code 报错。
- [ ] R1:重启后重新打开项目,页签正常(id 运行时生成,不依赖持久化)。
- [ ] R3:`FieldDetailDialog` 900px + 多列布局,备注在弹窗最末。
- [ ] R4:索引字段可排序、可选 ASC/DESC,DDL 含方向。
- [ ] R4:旧项目(fields 为 string[])打开默认 ASC,不报错。
- [ ] R4:索引名留空时前端展示自动生成名。
- [ ] R5:新建索引默认带一个空字段;删字段后索引引用同步移除;改字段 code 后索引引用同步改名。
- [ ] `vue-tsc --noEmit` + `cargo check` 通过。

## Out of Scope
- 表 code 跨表引用级联(字段不引用 table code,无此需求)。
- keep-alive 旧 path 清理(R1 用 id 后改 code 不产生旧 path)。
