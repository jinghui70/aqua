# 分组树 + 分组维护

## 背景
parent frontend-rework child。fe-arch 已建骨架(GroupTreeAside 占位)。本任务实现左侧分组树实体,是表编辑的入口。

## 目标
实现分组树:按分组组织表,分组/表的 CRUD,搜索,选中表 -> 打开表编辑标签。

## 包含(design.md §6.2)
- 分组树: 分组为父节点,表为子节点(el-tree)
- 分组维护: 新建/重命名/删除分组(删除需空或迁移表)
- 表维护: 新建表(选所属分组)/删除表/重命名
- 搜索: 按表名/分组名过滤
- 点击表节点 -> store.openTable -> 打开表编辑标签
- 可折叠

## 范围(不含)
- 拖拽排序/拖拽换组(§6.2 提及,本期简化,后续增强)
- 表/字段复制粘贴(后续)
- 表编辑内容(fe-table-editor)

## 验收标准
- [ ] GroupTreeAside 实现分组树(el-tree,分组>表 两层)
- [ ] 分组 CRUD: 新建/重命名/删除(删除非空分组提示)
- [ ] 表 CRUD: 新建(选分组)/删除/重命名
- [ ] 搜索过滤
- [ ] 点表 -> 打开表编辑标签(store.openTable)
- [ ] 操作走 ElMessageBox(无 window 弹窗)
- [ ] store 加 分组/表 增删改 actions
- [ ] pnpm build 通过

## 约束
- 直接改 store.currentProject.groups/tables(Pinia 响应式)
- code 唯一性校验(表 code 全局唯一,分组 code 唯一)
- unocss px + element-plus
