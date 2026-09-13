# 配置中心改为 Cmd+, 设置对话框

## Goal

把全屏路由页式的配置中心改成 Mac 原生风格的设置对话框:`Cmd + ,` 打开,左侧分类导航 + 右侧面板,不离开当前工作区。用户已确认范围:驱动管理并入对话框(第五个分类),并加 macOS 原生菜单"偏好设置…"项。

## Background(已核实事实)

### 现状

- **配置中心是全屏路由页**:`app/src/views/ConfigCenter.vue` 挂在路由 `/config`(`app/src/router/index.ts:38-42`),左侧 `el-menu` 导航(项目设置/数据源/业务类型/自动生成策略)+ 右侧四个 Panel 组件(`app/src/views/config/*.vue`,组件本身无路由依赖,可直接复用)。
- **入口仅工具栏一个按钮**:`app/src/layout/AppToolbar.vue:60` `router.push('/config')`。
- **驱动管理是独立小弹窗**:`app/src/components/DatabaseConfigDialog.vue`(el-dialog 680px,驱动安装/卸载 + 数据库显隐),入口是工具栏另一按钮(`AppToolbar.vue:68`)+ `ui.openDatabaseConfig()`(`app/src/stores/ui.ts:20-22`)。
- **原生菜单已有基建**:Rust 侧 `build_menu`(`src-tauri/src/lib.rs:19-58`)构建菜单,事件 emit "menu" → 前端 `useMenuActions.handle`(`app/src/composables/useMenuActions.ts`)分发。macOS 应用菜单(SubmenuBuilder "aqua" + about + quit)在 `lib.rs:47-53`,**目前没有 preferences 项**——Tauri 2 的 SubmenuBuilder 有原生 `.preferences()` 预置项(带系统标准 Cmd+, accelerator)。
- **对话框开关集中在 ui store**:`databaseConfigVisible` 等 8 个 visible ref + `anyDialogOpen` computed(`ui.ts:64-70`),菜单事件在对话框打开时被忽略(`useMenuActions.ts:26`)。
- **快捷键先例**:保存是 Rust 侧 MenuItemBuilder 带 `.accelerator("CmdOrCtrl+S")`(`lib.rs:22-24`),webview 内无全局 keydown 监听。

### 用户决策(已确认)

- 方案:Mac 风格大对话框(左侧分类 + 右侧面板),替代全屏路由页
- 驱动管理并入对话框作为第五个分类
- 加 macOS 原生菜单"偏好设置…"项(带 Cmd+,)

## Requirements

### R1:设置对话框组件

- 新建 `SettingsDialog.vue`(el-dialog 大尺寸,约 900×640):左侧分类导航(项目设置/数据源/业务类型/自动生成策略/驱动管理),右侧渲染对应 Panel 组件。
- 五个 Panel 全部复用现有组件,`DatabaseConfigDialog` 的表格主体抽成 `DriverPanel.vue` 移入 `views/config/`(去 el-dialog 壳)。
- 对话框记住上次选中的分类(会话内;持久化不要求)。
- 打开时若面板依赖当前项目(项目设置/数据源),无项目状态下相应面板显示空态提示而非报错。

### R2:入口与快捷键

- macOS 原生菜单:应用菜单加"偏好设置…"项(Tauri `.preferences()` 预置,自动带 Cmd+,),点击 emit `menu("app.settings")`。
- Windows/Linux:文件菜单加"设置"项(accelerator `CmdOrCtrl+,` → Windows 实际为 Ctrl+,)。
- 前端 `useMenuActions.handle` 加 `app.settings` 分支:打开设置对话框。
- 对话框打开时其余菜单事件仍被 `anyDialogOpen` 屏蔽(现有机制,把 settingsVisible 纳入 computed 即可)。

### R3:入口收敛

- 工具栏"配置"按钮改为打开设置对话框(不再路由跳转)。
- 工具栏"驱动管理"按钮移除(功能已并入对话框);若工具栏显得空,保留按钮位但行为统一为打开对话框并定位到驱动分类(实现取简单者:直接移除)。
- 移除 `/config` 路由与 `ConfigCenter.vue`;确认无其他引用后删除。

### R4:对话框行为细节

- `close-on-click-modal: false`(防误关,与 DatabaseConfigDialog 一致)。
- `destroy-on-close`(面板内的临时编辑状态不跨会话残留;各 Panel 自身有 watch 从 store 同步,重开即刷新)。
- Esc / 关闭按钮可关。

## Acceptance Criteria

- [ ] AC1:macOS 上按 Cmd+, 打开设置对话框,菜单栏"应用菜单"内可见"偏好设置…"项且带快捷键标注;点击菜单项同样打开。
- [ ] AC2:Windows/Linux 上 Ctrl+, 打开(文件菜单"设置"项)。
- [ ] AC3:五个分类均可切换,各面板功能(保存项目设置/增删改数据源/业务类型维护/自动生成策略/驱动安装卸载)与改造前等价。
- [ ] AC4:无项目打开时,设置对话框仍可打开,项目设置/数据源面板显示空态提示不报错;驱动管理/业务类型面板不依赖项目,正常可用(以现状行为为准,不引入新限制)。
- [ ] AC5:设置对话框打开时,其他菜单操作(保存/导出等)不执行(`anyDialogOpen` 覆盖);关闭后恢复。
- [ ] AC6:工具栏不再跳 `/config` 路由;`/config` 路径访问不再存在(重定向或 404 由 router 现状决定,不新增空白页)。
- [ ] AC7:vue-tsc 通过;现有交互回归:新建/打开/保存/导出不受影响。

## Out of Scope

- 各 Panel 内部的功能改造(纯搬家,不改逻辑)。
- 设置分类持久化记忆(跨启动)。
- webview 内额外全局快捷键监听(原生菜单 accelerator 已覆盖;输入框内打 `,` 不应误触——原生菜单 accelerator 系统级处理,无此问题)。
- Windows 专属的原生菜单样式微调(现有菜单体系什么样就什么样)。

## Key Decisions

- **D1(用户已定)**:Mac 风格对话框 + 驱动管理并入 + 原生菜单入口,三项范围已确认。
- **D2**:复用五个 Panel 组件不改内部逻辑,`DatabaseConfigDialog` 表格主体抽为 `DriverPanel.vue`;原 el-dialog 壳删除。
- **D3**:入口统一走 ui store(`settingsVisible` + `openSettings(panel?)`),与现有弹窗管理惯例一致(`ui.ts`)。
- **D4**:macOS 用 Tauri `.preferences()` 预置菜单项(系统标准位置与快捷键),Windows/Linux 走文件菜单 + `CmdOrCtrl+,` accelerator。
