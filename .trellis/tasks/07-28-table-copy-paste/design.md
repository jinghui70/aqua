# 设计

## 路由

`/group/:code` -> `GroupTablesPage.vue`（props: code = 分组 code）。点分组节点跳此路由，用页签（key `group:${code}`）。

## GroupTablesPage.vue（新，`app/src/views/GroupTablesPage.vue`）

- props: `code`（分组 code）
- 数据：`store.currentProject.tables.filter(t => t.group === code)`
- 工具栏：编辑分组 / 删除分组 / 新增表 / copy / paste / 删除表（只读隐藏编辑类）
- 表表格列：selection / 编码 / 名称 / 字段数 / 索引数 / 操作列（只读不可见）
  - 操作列：打开（router.push openTable）/ 编辑（openEditTable 弹框）/ 复制（duplicateTable）/ 删除（removeTable + closeTab）
- 点表行 -> `router.push(store.openTable(table))`
- 空 -> el-empty

## clipboard store（扩展，`app/src/stores/clipboard.ts`）

加 `tables` 槽（保留 fields）：`setTables`/`getTables`/`hasTables`，深拷贝，独立 project。

## store/project.ts

- `copyTables(tables)`: `clipboard.setTables(tables)`
- `pasteTables(groupCode): string[]`: `clipboard.getTables()` 每表新 id + code 冲突 `_COPY`/`_COPYn` + group=groupCode + 深拷贝 fields/indexes -> push tables
- `openGroup(code): string`: `openTab({key:\`group:${code}\`, title:分组名, path:\`/group/${code}\`})`
- `removeTable(code)`: 删表 + `closeTab(\`table:${id}\`)`（关页签）
- `removeGroup(code)`: 复用现有 `deleteGroup` 逻辑 + `closeTab(\`group:${code}\`)`（关分组页签）
- `duplicateTable` 保留（操作列调用）

## TableEditor.vue（改）

上部加表基本信息区：显示 code/name/comment（只读）+ "编辑"按钮（弹框 `openEditTable`，复用现有编辑逻辑）。group 用拖拽（不在信息区编辑）。

## GroupTreeAside.vue（改）

- `onNodeClick`：点分组 -> `router.push(store.openGroup(code))`；点表 -> `openTable`（不变）。
- 删 hover 菜单：`hoverNode` 逻辑 + 模板 :435-460 hover 按钮。
- `openEditGroup`/`openEditTable`/`deleteGroup`/`onDuplicate` 逻辑移到 GroupTablesPage（或保留 store 方法，GroupTablesPage 调用）。
- 拖拽 `moveTable` 保留（移表分组）。
- 展开/折叠用箭头（expand-on-click-node=false 已有）。

## 编辑弹框复用

- `openEditTable`（表 code/name/comment 编辑弹框）：GroupTablesPage 操作列"编辑" + TableEditor 信息区"编辑"调用。
- `openEditGroup`（分组 code/name 编辑弹框）：GroupTablesPage 工具栏"编辑分组"调用。
- 弹框组件从 GroupTreeAside 抽到独立组件或 store 共用（避免重复）。

## 数据流

```
表列表页选中表 -> copyTables -> clipboard.setTables(深拷贝)
[切项目]
点目标分组 -> 表列表页 -> paste -> clipboard.getTables -> 每表新 id + code 冲突 _COPY + group=目标 -> push tables
删表 -> removeTable -> closeTab(table:id)
删分组 -> removeGroup -> closeTab(group:code)
```

## 兼容/回归

- 字段级 clipboard 保留（fields 槽）。
- 拖拽移表分组保留。
- duplicateTable 逻辑不变（操作列调用）。
- 树点表打开编辑不变。
- 删分组时表处理沿用现有 deleteGroup 逻辑。
- **拖拽对已打开分组页面的影响**：GroupTablesPage 数据 = `computed filter(group===code)`，响应式。拖拽 `moveTable` 改 `table.group` 后，源分组页面表自动消失、目标分组页面（若打开）表自动出现；表编辑页不受影响（id 不变）。边界：拖拽时该表在源页面被选中（checkbox）-> 拖走后选中残留，需清理（watch 数据变化清空 selected，或拖拽后 clearSelection）。
