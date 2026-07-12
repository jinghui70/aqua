# 前端全量重做 (按 design.md §6)

## 背景与根因
P1 阶段前端 4 个任务被塞进单个 App.vue,做成简陋 demo:无菜单栏、无分组维护、无多表页签、无 bizType/Enum/数据源/数据集管理、表编辑过于简单。根因:实现时未读 design.md §6 完整 UI 需求。本 parent 任务推翻重做。

## 源需求(design.md §6 权威)
- §6.1 菜单栏: 文件/配置/导出/帮助
- §6.2 主界面: 分组树(可折叠/搜索) + 表编辑区 + 状态栏
- §6.3 表编辑页: 4 Tab(fields/index/java/json),多表同时打开
- §6.4 数据集管理
- §6.5 bizType 管理
- §6.6 Enum 管理
- §6.7 数据源配置
- §6.8 导入向导(4步)
- §6.9 导出(DDL/diff/StrConst)

## 架构决策(已定)
- **路由**: vue-router。表编辑/bizType/Enum/数据集 = 路由页签;数据源/导入/导出 = 弹窗
- **状态**: Pinia(project/openedTabs/datasource/dataset)
- **布局**: 菜单栏 + 左分组树 + 右多标签工作区 + 状态栏(参照 DataGrip/DBeaver)
- **keep-alive**: 缓存已打开表编辑页
- **样式**: unocss px 单位 + element-plus
- **弹窗**: 一律 ElMessageBox/el-dialog,禁用 window.prompt/confirm/alert

## 任务地图(child)
1. fe-arch — 架构骨架(路由/Pinia/布局/菜单栏)【阻塞后续】
2. fe-group-tree — 分组树 + 分组维护
3. fe-table-editor — 表编辑多标签页 + 4-Tab
4. fe-biztype — bizType 管理页
5. fe-enum — Enum 管理页
6. fe-datasource — 数据源配置(弹窗)
7. fe-import-wizard — 导入向导(4步弹窗)
8. fe-export — 导出(DDL/diff/StrConst 弹窗)
9. fe-dataset — 数据集管理页

依赖: fe-arch 先做(建立路由/布局/Pinia),其余可并行。fe-table-editor 依赖 fe-group-tree(树选中驱动打开表)。

## 跨 child 验收
- [ ] 菜单栏可用,各菜单项路由/弹窗正确
- [ ] 分组树增删改 + 表挂分组
- [ ] 多表同时打开(标签页 keep-alive)
- [ ] 4 Tab 完整(fields 行内+弹窗编辑/index/java 预览配置/json 预览)
- [ ] bizType/Enum/数据源/数据集 管理可用
- [ ] 导入向导 4 步 + 导出 3 项(预览+复制+下载)
- [ ] 全程无 window 原生弹窗
- [ ] pnpm build 通过

## 现有可复用
- Tauri commands: project_open/save/validate, generate_ddl/java, test_connection/import_from_db
- 缺失 command 各 child 按需补(标注到 child)
- types/schema.ts, useTauri.ts(保留扩展)
