# fe-arch 实现计划

## Rust 侧(原生菜单)
1. [ ] src-tauri/src/lib.rs — setup 构建原生菜单(文件/配置/导出/帮助)+ on_menu_event emit "menu" 事件

## 前端侧
2. [ ] 确认 vue-router/pinia 依赖(package.json 已有)
3. [ ] stores/project.ts — Pinia project store(迁移 useProject:currentProject/path/openedTabs/actions)
4. [ ] router/index.ts — 路由表 + 占位 views
5. [ ] views/ 占位页(Welcome/TableEditor/BizTypeManage/EnumManage/DatasetManage)
6. [ ] composables/useMenuActions.ts — listen('menu') -> 路由/弹窗占位/store
7. [ ] layout/TabWorkspace.vue — el-tabs 多标签 + router-view + keep-alive
8. [ ] layout/GroupTreeAside.vue — 左树占位
9. [ ] layout/StatusBar.vue — 状态栏
10. [ ] layout/AppLayout.vue — 组装(aside + workspace + statusbar,无菜单栏)
11. [ ] App.vue — 布局容器 + useMenuActions 挂载
12. [ ] main.ts — 挂 router + pinia

## 验证
- cargo build -p aqua 通过(原生菜单)
- pnpm build 通过
- pnpm dev: 窗口顶部/菜单栏显示原生菜单;点"配置>业务类型管理"开路由标签;标签可关闭切换
