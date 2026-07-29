# 表复制粘贴跨项目

## Goal

表 copy/paste 跨项目，支持多表。点分组节点打开表列表页（空间开阔），工具栏 copy/paste + 操作列编辑/复制/删除；树节点去 hover/右键菜单，编辑移页面；删表/删分组关对应页签。

## Background

- `useClipboardStore`（`app/src/stores/clipboard.ts`）：字段级剪贴板（跨项目）。
- `duplicateTable`（`project.ts:473`）：同项目复制表（`_COPY`/`_COPYn`，新 id，深拷贝）。
- `GroupTreeAside.vue`：树节点 hover 菜单（分组:修改/删除 :435-444；表:复制/修改/删除 :446-460）。`onNodeClick` 点表打开编辑。`deleteGroup`/`openEditGroup`/`openEditTable` hover 入口。拖拽 `moveTable`（移表分组）。el-tree `expand-on-click-node=false`。
- `TableEditor.vue`：表编辑页（字段/索引/DDL/Java tab），无表基本信息（code/name/comment）编辑区。
- 路由 /table/:id 表编辑。

## Requirements

- R1 表列表页（新，`/group/:code`）：点分组节点 -> 打开该分组的表列表页。
- R2 表列表页工具栏：编辑分组 / 删除分组 / 新增表 / copy / paste / 删除表（只读时编辑类按钮隐藏）。
- R3 表表格：selection / 编码 / 名称 / 字段数 / 索引数 / 操作列（打开/编辑/复制/删除，只读不可见）；点表行打开表编辑。
- R4 copy：选中表存剪贴板（深拷贝，独立 project，跨项目保留）。
- R5 paste：粘贴剪贴板表到当前分组，新 id，code 冲突 `_COPY`/`_COPYn`。
- R6 duplicate：操作列"复制"按钮（`duplicateTable` 一键 `_COPY`，单表）。
- R7 表基本信息编辑（code/name/comment；group 用拖拽）：两入口 -- 表编辑页上部信息区（显示+编辑）+ 表列表页操作列"编辑"弹框。
- R8 分组基本信息编辑（code/name）：分组页面工具栏"编辑分组"弹框。
- R9 分组删除：分组页面工具栏"删除分组"（复用 `deleteGroup` 逻辑）。
- R10 删表 -> 关闭该表页签（`closeTab(table:id)`）。
- R11 删分组 -> 关闭该分组页签（`closeTab(group:code)`）。
- R12 树节点去 hover/右键菜单（只点：点分组开表列表页，点表开编辑）；展开/折叠用箭头；拖拽移表分组保留。
- R13 点分组打开表列表页用页签（key `group:${code}`，复用 openTab）。

## Key Decisions

- D1 点分组 -> 表列表页（表表格 + 工具栏）。
- D2 树去 hover/右键菜单，编辑移页面。
- D3 paste code 冲突 `_COPY` 后缀（与 duplicateTable 一致）。
- D4 剪贴板独立 project（跨项目，clipboard store 加 table 槽）。
- D5 duplicate 保留在操作列（一键 `_COPY`，单表快捷）。
- D6 表基本信息两入口（表编辑页信息区 + 操作列编辑弹框）；group 拖拽。
- D7 分组编辑/删除在分组页面工具栏。
- D8 删表/删分组关对应页签。

## Acceptance Criteria

- [ ] 点分组 -> 表列表页（页签）；展开/折叠用箭头；点表开编辑。
- [ ] 表列表页选中多表 -> copy -> 切项目 -> 点目标分组 -> paste -> 表粘贴（新 id，code 冲突 `_COPY`）。
- [ ] 操作列"复制" -> 单表 duplicate（`_COPY`）。
- [ ] 操作列"编辑" -> 弹框改 code/name/comment；表编辑页上部信息区也可编辑。
- [ ] 分组页面工具栏"编辑分组"/"删除分组"可用。
- [ ] 删表 -> 关闭该表页签；删分组 -> 关闭分组页签。
- [ ] 树无 hover/右键菜单；拖拽移表分组仍可用。
- [ ] 只读模式：操作列 + 编辑类按钮隐藏。

## Out of Scope

- 字段级 copy/paste（已有）。
- 表数据（dataset）复制。
- 删分组时表的处理逻辑（沿用现有 `deleteGroup`，不在本任务改）。
