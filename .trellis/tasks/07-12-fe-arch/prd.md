# 前端架构骨架 (路由/Pinia/布局/菜单栏)

## 背景
parent 07-12-frontend-rework 第一个 child,阻塞后续。建立前端整体架构,后续 child 往骨架填业务。

## 目标
搭建路由 + Pinia + IDE 风格布局 + 菜单栏骨架,后续页面挂进来即可。

## 包含
- vue-router 安装配置 + 路由表(表编辑/bizType/enum/dataset 路由占位)
- Pinia 安装配置 + project store(替换现有 useProject composable)
- 布局组件: MenuBar / GroupTreeAside(占位) / TabWorkspace(多标签) / StatusBar
- 菜单栏: 文件/配置/导出/帮助(§6.1),项触发路由跳转或弹窗(占位)
- 多标签工作区: el-tabs 可关闭,标签驱动路由 + keep-alive
- App.vue 重构为布局容器

## 范围(不含)
- 分组树实体逻辑(fe-group-tree)
- 表编辑内容(fe-table-editor)
- 各管理页/弹窗内容(各 child)
- 本任务只搭骨架 + 占位,能跑通路由切换 + 标签开关

## 验收标准
- [ ] vue-router + Pinia 装配
- [ ] router: 路由表(/table/:code, /biztype, /enum, /dataset 占位页)
- [ ] stores/project.ts: Pinia project store(currentProject/openedTabs/actions)
- [ ] layout: MenuBar/TabWorkspace/StatusBar + 左侧 aside 占位
- [ ] 菜单栏 §6.1 结构,项可点(路由跳转/弹窗占位)
- [ ] 多标签: 打开/关闭/切换 + keep-alive
- [ ] App.vue = 布局容器
- [ ] pnpm build 通过

## 约束
- unocss px 单位, element-plus, 无 window 原生弹窗
- 保留 types/schema.ts, useTauri.ts
- useProject composable 迁移到 Pinia store(其余 child 依赖 store)
