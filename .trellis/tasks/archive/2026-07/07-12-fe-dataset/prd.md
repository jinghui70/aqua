# 数据集管理页

## 背景
parent frontend-rework 最后 child(P3)。菜单"配置>数据集管理"路由页。§6.4。
后端 dataset 模块(SQLite 容器)已有,但数据行 CRUD/从库导入的 Tauri command 未做。

## 目标(本期)
数据集管理页框架: 数据集选择 + 表树(显示行数)+ 数据网格 UI。
数据实际读写(SQLite 持久化/从库导入行)标注为后续后端任务。

## 包含
- 数据集下拉(新建/删除)+ 隐藏无数据表开关
- 表树(按分组,显示行数占位)
- 数据网格区(表格 UI + 新增行/从库导入/导出/清空 按钮占位)

## 范围(不含,标注后续)
- SQLite 数据行持久化 command(dataset-commands 后端任务)
- 从库导入行数据(异构字段映射)
- 虚拟滚动(大数据量)

## 验收标准
- [ ] DatasetManage.vue: 数据集选择 + 表树 + 数据网格框架
- [ ] 表树按分组展示(复用 currentProject.groups/tables)
- [ ] 数据网格显示表字段列(表头)
- [ ] 操作按钮占位(标注 dataset-commands 后端任务)
- [ ] pnpm build 通过

## 约束
- 本期 UI 框架为主,数据读写待后端 command
- element-plus el-tree + el-table, unocss px
