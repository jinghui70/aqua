# Implement 计划

## 已完成(本任务已改,未提交)

1. **store readOnly**(`stores/project.ts`):`readOnly` ref + `openProject`(true)/`newProject`/`closeProject`(false)/`toggleReadOnly`。
2. **Welcome 去导入按钮**:删"从数据库导入"卡片 + `handleImport`。

## 待实现(按依赖顺序)

### 3. 工具栏
- 新建 `app/src/layout/AppToolbar.vue`(导入/导出/加解锁/配置中心/数据集/驱动管理)。
- `AppLayout` 有项目时顶部引入 `AppToolbar`(`el-splitter` 前)。

### 4. 驱动改名
- `DatabaseConfigDialog` title "数据库配置" -> "驱动管理"。

### 5. 只读传播
- GroupTreeAside:新建分组/表按钮、删除 disabled
- FieldsTab:addField/removeField/openDetail/copy disabled
- IndexTab:addIndex/删除 disabled
- FieldDetailDialog:表单字段 disabled
- BizTypeManage/DatasetManage:增删改 disabled
- ProjectSettingsDialog:表单 disabled(配置中心迁移前临时)

### 6. 配置中心 /config
- `router/index.ts` 加 `/config` -> `ConfigCenter.vue`
- `ConfigCenter.vue`:左侧返回 + el-menu 导航(项目设置/数据源/业务类型)+ 动态组件
- `ProjectSettingsPanel.vue`(从 ProjectSettingsDialog 抽表单)
- `DataSourcePanel.vue`(从 DataSourceDialog 抽)
- `BizTypePanel.vue`(从 BizTypeManage 抽)
- AppToolbar 配置中心按钮跳 /config

### 7. 数据集 /dataset
- `DatasetManage.vue` 加左侧返回按钮(`router.push('/')`)

### 8. 关闭应用 dirty 检查
- `App.vue` setup listen `getCurrentWindow().onCloseRequested`:dirty 时 `event.preventDefault()` + `await store.confirmIfDirty()`;取消则不关,保存/不保存则 `getCurrentWindow().close()`。

### 9. 全局枚举全删
- 先确认 generators 枚举逻辑(grep enum 生成)
- `schema/field.rs`:FieldEnum 只 Inline,`enum_ref` 改 `Option<InlineEnum>`
- `schema/enum_def.rs`:EnumDefine 删,InlineEnum 保留
- `schema/mod.rs`/`validate.rs`:Project.enums 删,enums 校验删
- `diff/mod.rs`:enums diff 删
- generators:枚举生成删/改(视确认)
- 前端:EnumManage.vue 删 + /enum 路由删 + FieldDetailDialog 枚举编辑只内联 + types/schema.ts enums 删

### 10. 菜单精简
- `src-tauri/src/lib.rs` build_menu:删配置/导出子菜单;文件删导入,保留新建/打开/最近/保存/另存/关闭;帮助保留
- `useMenuActions.ts`:删 config.*/export.*/file.import case

## 验证

- `cargo test -p aqua-core` + `cargo clippy -p aqua-core -p aqua`
- `cargo build -p aqua`
- `cd app && pnpm vue-tsc --noEmit && pnpm vite build`
- 手动:打开项目(只读)-> 解锁 -> 编辑 -> 关闭窗口(提示保存);配置中心三项;数据集;驱动管理;导出;菜单

## 风险点

- 步骤 6(配置中心内容迁移)最大,保持逻辑仅改容器
- 步骤 9(全局枚举删)跨 Rust/前端,先确认 generator
- 步骤 5(只读传播)逐项检查
