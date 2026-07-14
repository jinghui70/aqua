# 表编辑页改进 - 执行计划

## 实现顺序(底层 -> 上层,先后端再前端)

### 阶段 A:后端索引结构 + DDL
- [x] A1 `schema/table.rs`:加 `Direction` enum(UPPERCASE serde)、`IndexField` struct(code+direction)、自定义 `Deserialize` 兼容旧 `string[]`(纯字符串默认 ASC)。`Index.fields` 改 `Vec<IndexField>`。Table 不变。
- [x] A2 `generators/ddl/index.rs`:`generate_index` 字段映射为 `CODE DIRECTION` join;`auto_index_name` 用 `IndexField.code` join。
- [x] A3 `diff/mod.rs`:`index_key` 含方向(方向不同 = 变更)。
- [x] A4 验证:`cargo check -p aqua-core` + 跑 diff/ddl 单测(若有)。

### 阶段 B:前端 R1 表 id 解耦
- [x] B1 `types/schema.ts`:`IndexField`、`Index.fields: IndexField[]`、`Table.id: string`。
- [x] B2 `stores/project.ts`:`openTable` key/path 用 `table.id`(title 仍用 code);`openProject` 加载后 `.map(t => ({...t, id: crypto.randomUUID()}))`;`addTable` 生成 id;加 `updateTable(id, code, name)`(用 id 定位,code 唯一校验除自己)。
- [x] B3 `router/index.ts`:`/table/:code` -> `/table/:id`。
- [x] B4 `TableEditor.vue`:`tables.find(t => t.id === props.id)`。
- [x] B5 `GroupTreeAside.vue`:表对话框 code 去 `disabled`;`tableEditingCode` -> `tableEditingId`;`confirmTableDialog` edit 分支调 `store.updateTable(id, code, name)`;`treeData` 表节点 `id` 改 `table:${t.id}`(分组节点保持 `group:${g.code}`)。

### 阶段 C:前端 R3 弹窗
- [x] C1 `FieldDetailDialog.vue`:width `600px` -> `900px`,基本区表单多列布局;备注从基本区移到弹窗最末(业务类型区后)。

### 阶段 D:前端 R4 索引 UI
- [x] D1 `IndexTab.vue`:索引字段从 `el-select multiple` 改成行列表(每行:字段 select + 方向 select ASC/DESC + 拖拽柄/删除),可增删行、拖拽排序。复用 FieldsTab 的 Sortable.js 模式或上下移按钮。

### 阶段 E:整体验证
- [x] E1 `cd app && pnpm exec vue-tsc --noEmit`。
- [ ] E2 手动(Tauri dev):改表 code 页签不失效且显示新 code;改分组 code 表归属同步;索引加字段+选方向,DDL 预览含 ASC/DESC;旧项目打开索引默认 ASC 不报错;字段弹窗 900px 备注在末;索引名留空展示自动生成名。

### 阶段 F:本会话收尾(任务范围外补丁,随本任务提交)
- [x] F1 `TableEditor.vue`:去掉表头"修改"按钮 + 编辑表对话框(表 code 编辑统一走树节点"改")。
- [x] F2 `DdlTab/JavaTab/JsonTab`:加 `active` prop,切回 tab 时重新生成,同步字段/索引改动。
- [x] F3 `GroupTreeAside.vue`:hover 操作栏横向 -> 纵向(等宽居中,清 el-button 相邻 margin 错位)。
- [x] F4 删表关路由:`store.deleteTable` 用 `table:${id}` 关页签(原误用 code)+ 透传 `closeTab` 返回路径;`GroupTreeAside` 接收后 `router.push`。
- [x] F5 测试/格式:fixture `valid-full.json` 索引改新格式(修 `test_serde_roundtrip`);`cargo fmt` 全量。

### 阶段 G:R5 索引与字段联动
- [x] G1 `IndexTab.vue`:`addIndex` 默认带一个空字段 `{ code: "", direction: "ASC" }`。
- [x] G2 `stores/project.ts`:加 `renameFieldCode(tableId, old, new)` + `removeFieldFromIndexes(tableId, code)`,按 tableId 定位表,遍历 indexes 级联。
- [x] G3 `FieldsTab.vue`:props 加 `tableId`;`removeField` 删后级联;`onCodeChange` 用 `@focus` 缓存旧 code,变更后级联;Dialog 透传 `tableId`。
- [x] G4 `FieldDetailDialog.vue`:props 加 `tableId`;`save` 用 `props.field.code`(assign 前旧值)对比 `draft.code` 级联。
- [x] G5 `TableEditor.vue`:FieldsTab 传 `:table-id`。
- [x] G6 验证:`vue-tsc --noEmit` 通过(后端未动,门禁沿用)。

### 阶段 H:字段列 UI 细化(用户迭代)
- [x] H1 `FieldsTab.vue`:表头 code/prop 中文化(编码/属性名)。
- [x] H2 code 输入规范化:`onCodeInput` 大写 + 仅留 `[A-Z0-9_]` + 去开头数字;`FieldsTab` 与 `FieldDetailDialog` 两端统一。
- [x] H3 code 联动 prop 从 `@change` 改 `@input`(实时);`@change` 只留索引级联。
- [x] H4 `FieldsTab.vue`:操作列 `fixed="right"`,横向滚动不跟随。

## 验证命令
- 后端:`cargo check -p aqua-core`
- 前端:`cd app && pnpm exec vue-tsc --noEmit`
- 手动:`pnpm tauri dev`(已在运行则 HMR)

## 风险点 / 回滚
- **IndexField 自定义 Deserialize**(A1):必须测旧 JSON(`fields: ["a","b"]`)能加载为默认 ASC。回滚:git revert `table.rs`,但新版本保存的 `{code,direction}` 格式旧版本读不了--项目早期数据少,从 git 旧版恢复项目文件。
- **Table.id 加载补全**(B2):所有加载路径(`openProject`)必须补 id,否则页签 key 为 undefined。review 时确认无遗漏。
- **route 改 /:id**(B3):同步 `openTable` path、`closeTab` 返回 path。遗漏会导致跳转 404。
- **IndexTab UI 重构**(D1):改动最大,易出响应式问题(直接操作 `table.indexes` 引用,同现状)。

## start 前检查
- [ ] prd.md / design.md / implement.md 三件齐备且用户已 review。
- [ ] 无 blocking open question。
