# Design - 配置体系重组 + 工作流优化

## 1. 工具栏(AppToolbar)

- 新建 `app/src/layout/AppToolbar.vue`,AppLayout 有项目时顶部(`<AppToolbar />` 在 `el-splitter` 前),`flex-shrink-0`。
- 按钮:导入(`ui.openImport`,disabled=readOnly)/ 导出(`ui.openExport('ddl')`)/ 加解锁(`store.toggleReadOnly`)/ 配置中心(`router.push('/config')`)/ 数据集(`router.push('/dataset')`)/ 驱动管理(`ui.openDatabaseConfig`)。

## 2. 只读机制

- `stores/project.ts`:加 `readOnly` ref(已改)。`openProject` 设 true、`newProject`/`closeProject` 设 false、`toggleReadOnly` 切换。
- 传播(编辑点 `disabled="store.readOnly"`):
  - AppToolbar 导入按钮
  - GroupTreeAside:新建分组/表按钮、删除
  - FieldsTab:addField/removeField/openDetail/copy
  - IndexTab:addIndex/删除
  - FieldDetailDialog:表单字段 disabled
  - BizTypePanel/DatasetPanel:增删改按钮
  - ProjectSettingsPanel:表单 disabled
- 不受限:导出、数据源(外置)、驱动管理(应用级)。
- 解锁后编辑触发现有 dirty watch;保存后 dirty=false(readOnly 不变)。

## 3. 配置中心 /config

- `router/index.ts` 加 `/config` -> `ConfigCenter.vue`。
- `ConfigCenter.vue`:左侧返回按钮(`router.push('/')`)+ 左侧 el-menu(项目设置/数据源/业务类型)+ 右侧动态组件切换。
- 内容迁移(抽面板组件,复用现有逻辑):
  - `ProjectSettingsPanel.vue`:从 ProjectSettingsDialog 抽表单(basePackage/name)
  - `DataSourcePanel.vue`:从 DataSourceDialog 抽列表+编辑
  - `BizTypePanel.vue`:从 BizTypeManage 抽列表+编辑
- 原 ProjectSettingsDialog/DataSourceDialog 可删(内容进面板);BizTypeManage 改 BizTypePanel 或保留页内联。

## 4. 数据集 /dataset

- DatasetManage.vue 加左侧返回按钮(`router.push('/')`)。内容不变。

## 5. 驱动管理

- DatabaseConfigDialog title "数据库配置" -> "驱动管理"。

## 6. 全局枚举全删

- `schema/field.rs`:FieldEnum 只留 Inline;`enum_ref` 改 `Option<InlineEnum>`(或删 FieldEnum,直接 InlineEnum)。
- `schema/enum_def.rs`:EnumDefine 删,InlineEnum 保留。
- `schema/mod.rs`/`validate.rs`:Project.enums 字段删,enums 校验删。
- `diff/mod.rs`:enums diff 删。
- `generators`:确认是否有全局枚举生成(Java enum 类?),有则删/改。
- `import/from_db.rs`:enum_ref: None(已 None,无改动)。
- 前端:EnumManage.vue 删 + 路由 /enum 删 + FieldDetailDialog 枚举编辑改只内联 + types/schema.ts enums 删。

## 7. 菜单精简

- `src-tauri/src/lib.rs` build_menu:删"配置""导出"子菜单。文件保留(新建/打开/最近/保存/另存/关闭;导入移工具栏,菜单删)。帮助保留(指南/关于)。macOS 应用菜单保留。
- `useMenuActions.ts`:删 config.*/export.*/file.import case(移工具栏)。file.*(新建/打开/保存/另存/关闭/最近)+ help.* 保留。

## 8. 关闭应用检查 dirty

- 关闭项目:closeProject 已 confirmIfDirty(保留,验证有效)。
- 关闭应用:`App.vue` 或 `AppLayout` setup 时 listen `getCurrentWindow().onCloseRequested`:
  - if `store.dirty`: `event.preventDefault()` + `await store.confirmIfDirty()`;若用户取消(return false)则保持打开;若保存/不保存则调用 `getCurrentWindow().close()` 继续关闭。
  - if !dirty:直接关闭(不 prevent)。
- Tauri 2.x: `@tauri-apps/api/window` 的 `getCurrentWindow().onCloseRequested(callback)`,callback 的 event 有 `preventDefault()`。
- 注意:confirmIfDirty 内部用 ElMessageBox(同步等待),async callback 内 await 可行。

## 影响面

- 前端:AppToolbar(新)、AppLayout、ConfigCenter(新)、ProjectSettingsPanel/DataSourcePanel/BizTypePanel(新/改)、DatasetManage(改)、DatabaseConfigDialog(改名)、EnumManage(删)、FieldDetailDialog(改)、router、useMenuActions、stores/project(readOnly,已改)、Welcome(已改)、App.vue/AppLayout(onCloseRequested)。
- Rust:schema(field/enum_def/validate/mod)、diff、generators(枚举,待确认)、import(无改)、lib.rs(菜单)。

## 风险

- 只读传播遗漏点(某些编辑未禁用)-> 验收逐项检查。
- 配置中心内容迁移破坏现有功能(数据源/业务类型 CRUD)-> 迁移保持逻辑,仅改容器。
- 全局枚举删影响 generator/diff -> 实现前确认 generator 枚举逻辑。
- onCloseRequested 与 ElMessageBox 异步交互 -> 测试取消/保存/不保存三路径。

## 验证

- Rust:cargo test + clippy(schema/diff/generator 变更)。
- 前端:vue-tsc + vite build。
- 手动:打开项目(只读)-> 解锁 -> 编辑 -> 关闭窗口(提示保存);配置中心三项;数据集;驱动管理;导出;菜单精简。
