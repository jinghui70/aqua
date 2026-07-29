# 实现计划

## 步骤

1. `app/src/stores/clipboard.ts`：加 `tables` 槽（setTables/getTables/hasTables），保留 fields。
2. `app/src/stores/project.ts`：加 `copyTables`/`pasteTables(groupCode)`/`openGroup(code)`；`removeTable` 加 `closeTab(table:id)`；`removeGroup`/`deleteGroup` 加 `closeTab(group:code)`；return 导出。
3. `app/src/router/index.ts`：加 `/group/:code` -> `GroupTablesPage.vue`（props: true）。
4. 新建 `app/src/views/GroupTablesPage.vue`：
   - 工具栏：编辑分组/删除分组/新增表/copy/paste/删除表（只读隐藏编辑类）
   - 表表格：selection/编码/名称/字段数/索引数/操作列（打开/编辑/复制/删除，只读不可见）
   - 点行 openTable；操作列编辑 openEditTable 弹框、复制 duplicateTable、删除 removeTable
5. 抽编辑弹框：`openEditTable`/`openEditGroup` 从 GroupTreeAside 抽到独立组件或 store 共用（GroupTablesPage + TableEditor 复用）。
6. `app/src/views/TableEditor.vue`：上部加表基本信息区（显示 code/name/comment + 编辑按钮 -> openEditTable 弹框）。
7. `app/src/layout/GroupTreeAside.vue`：
   - `onNodeClick` 点分组 -> `router.push(store.openGroup(code))`；点表 -> openTable。
   - 删 hover 菜单（hoverNode + :435-460）；编辑/删除逻辑移 GroupTablesPage 或保留 store 方法。
   - 拖拽 moveTable 保留。
8. 验证。

## 验证命令

- `pnpm -C app exec vue-tsc --noEmit`
- `pnpm dev` 手测：
  - 点分组 -> 表列表页（页签）；展开/折叠箭头；点表开编辑。
  - 选中多表 copy -> 切项目 -> 点目标分组 paste -> 表粘贴（新 id，code 冲突 _COPY）。
  - 操作列：打开/编辑（弹框）/复制（_COPY）/删除（关页签）。
  - 分组页面工具栏：编辑分组/删除分组（关页签）。
  - 表编辑页上部信息区 + 编辑按钮。
  - 拖拽移表分组。
  - 拖拽表到另一分组 -> 源分组页面表消失、目标分组页面表出现（响应式）；选中残留已清理。
  - 只读：操作列 + 编辑类隐藏。

## 风险点 / 回滚

- 编辑弹框抽离复用（GroupTreeAside -> 独立组件），勿漏调用点。
- removeTable/removeGroup 关页签：确认 closeTab key 与 openTab 一致（table:id / group:code）。
- paste code 冲突 _COPY 逻辑与 duplicateTable 对齐。
- 表列表页点行 openTable vs selection checkbox 不冲突。
- TableEditor 信息区编辑按钮弹框与操作列编辑弹框复用同一弹框。
- 去 hover 菜单后，GroupTreeAside 原有 hover 操作（编辑/删除）全部移到页面，勿漏。
- 回滚：git revert。
