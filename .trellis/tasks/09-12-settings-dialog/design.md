# 技术设计:配置中心改为 Cmd+, 设置对话框

## 1. 结构总览

```
原生菜单(Rust build_menu)                     前端
┌──────────────────────────────┐              ┌─────────────────────────────────┐
│ macOS: 应用菜单 .preferences()│──emit menu──▶│ useMenuActions.handle            │
│ Win:   文件菜单"设置"项        │  app.settings │   → ui.openSettings()           │
└──────────────────────────────┘              │      ↓                           │
                                              │ SettingsDialog.vue (el-dialog)   │
工具栏"配置"按钮 ── ui.openSettings() ────────▶│ ┌──────┬──────────────────────┐ │
                                              │ │ 导航  │ Panel(复用现有组件)   │ │
                                              │ │ 5 项  │ ProjectSettingsPanel   │ │
                                              │ │      │ DataSourcePanel        │ │
                                              │ │      │ BizTypePanel           │ │
                                              │ │      │ AutoGenStrategyPanel   │ │
                                              │ │      │ DriverPanel(新抽)      │ │
                                              │ └──────┴──────────────────────┘ │
                                              └─────────────────────────────────┘
```

删除:路由 `/config`、`views/ConfigCenter.vue`、`components/DatabaseConfigDialog.vue`、工具栏驱动管理按钮。

## 2. Rust 侧:菜单(`src-tauri/src/lib.rs`)

### macOS 应用菜单(`lib.rs:47-53` 现有 builder)

SubmenuBuilder 加 `.preferences()`——Tauri 2 预置项,id 固定为 `app.settings`,系统自动绑 Cmd+, 并本地化菜单文案(macOS 上显示"偏好设置…")。点击走统一 menu 事件链。

```rust
#[cfg(target_os = "macos")]
let builder = {
    let app_menu = SubmenuBuilder::new(app, "aqua")
        .about(None)
        .preferences()          // 新增:Cmd+, 打开设置,emit id "app.settings"
        .separator()
        .quit()
        .build()?;
    builder.item(&app_menu)
};
```

### Windows/Linux:文件菜单加设置项

```rust
// 保存 MenuItemBuilder 的同款做法:需要 accelerator,单独构造
#[cfg(not(target_os = "macos"))]
let settings = MenuItemBuilder::new("设置")
    .id("app.settings")
    .accelerator("CmdOrCtrl+Comma")
    .build(app)?;
```

注意 accelerator 字符串:`,` 的合法写法是 `Comma`(`CmdOrCtrl+Comma`);Tauri 2 支持 `Comma` 键名。挂在文件菜单末尾(quit 之前)。

### 事件链

`.preferences()` 点击时 Tauri emit 的 menu 事件 id 为 `app.settings`(Tauri 2 预置 preferences 项默认 id)——需在实现时以实际 emit 验证为准,若 id 不同(如 `preferences`)则统一在 Rust 侧 on_menu_event 分发时映射为 `app.settings` 再 emit。

## 3. 前端

### ui store(`app/src/stores/ui.ts`)

```ts
// 设置对话框
const settingsVisible = ref(false);
const settingsPanel = ref<SettingsPanel>("settings");
function openSettings(panel: SettingsPanel = "settings") {
  settingsPanel.value = panel;
  settingsVisible.value = true;
}
```

- `SettingsPanel` 类型:`"settings" | "datasource" | "biztype" | "strategy" | "driver"`,导出供 dialog 与菜单用。
- `anyDialogOpen` computed 加入 `settingsVisible.value`。
- 删除 `databaseConfigVisible` / `openDatabaseConfig`(DatabaseConfigDialog 移除后无消费方)。

### useMenuActions(`useMenuActions.ts`)

```ts
case "app.settings":
  ui.openSettings();
  break;
```

注意 handle 入口的 `if (ui.anyDialogOpen) return;` 会拦截"设置已打开时再按 Cmd+,":行为可接受(模态约定),不特判。

### SettingsDialog.vue(新,`app/src/components/`)

- `el-dialog v-model="ui.settingsVisible" width="900px" top="6vh" :close-on-click-modal="false" destroy-on-close`,title "设置",`draggable` 对齐现有弹窗惯例(DatabaseConfigDialog 有 draggable,但设置类大对话框 Mac 上通常不可拖;取项目惯例 → draggable)。
- body 布局:flex 左导航(约 160px,el-menu 与 ConfigCenter 同款)+ 右面板区(`flex-1 overflow-auto p-12`)。
- `el-dialog` body 默认有 padding,设置类对话框需要自定义:用 `:deep(.el-dialog__body)` 归零 padding,让面板自己控制(对齐 element-plus 设置类弹窗常见做法)。
- 面板切换:`v-if` 链,与 ConfigCenter.vue:29-32 同款,只是组件换成五个。
- 打开时同步 `activePanel = ui.settingsPanel`(支持定位到指定分类)。
- 高度:body 定高 `h-560` 左右,面板内部各自滚动(现有 Panel 在 ConfigCenter 里就是 overflow-auto 容器的子元素,布局习惯不变)。

### DriverPanel.vue(新,`app/src/views/config/`)

- 从 `DatabaseConfigDialog.vue:66-110` 抽表格主体(驱动列表 + 安装/卸载 + 显隐 + 说明文案),script 逻辑原样搬运。
- 去掉 el-dialog 壳与 ui store 依赖(表格主体不依赖 databaseConfigVisible)。

### AppToolbar.vue

- "配置"按钮 → `ui.openSettings()`(图标不变)。
- "驱动管理"按钮删除(功能在设置对话框第五分类)。

### 路由与旧文件清理

- `router/index.ts` 删 `/config` 路由。
- 删 `views/ConfigCenter.vue`、`components/DatabaseConfigDialog.vue`。
- grep 确认无残余引用(`router.push('/config')`、`openDatabaseConfig`、`databaseConfigVisible`)。

## 4. 兼容性

| 面 | 影响 |
|---|---|
| Panel 组件 | 纯复用,不改内部;从路由页容器搬进 dialog 容器,布局上下文一致(都是 overflow-auto + padding) |
| 菜单事件 | 新增 `app.settings` id,现有 id 不动;`anyDialogOpen` 拦截逻辑天然覆盖 |
| `/config` 深链 | 桌面应用无外部深链场景,删除路由无兼容负担 |
| Windows | `CmdOrCtrl+Comma` 系统映射为 Ctrl+,;菜单文案"设置"(不做 per-OS 文案分支) |

## 5. 权衡记录

- **`.preferences()` 预置 vs 自建 MenuItem**:预置项在 macOS 上位置正确(应用菜单第二项)、快捷键系统级、文案自动本地化;自建则三平台统一但 Mac 上位置不标准。按平台分用两者,各取所长。
- **destroy-on-close vs 常驻**:面板内有表单临时态(如数据源表单填了一半),常驻对话框在误关后重开能保留——但跨分类切换用 v-if 本来就会销毁,保留意义有限;destroy-on-close 换来"重开必与 store 同步"的确定性,选后者。
- **保留工具栏驱动管理按钮 vs 移除**:移除。按钮越多入口越散;驱动管理是低频操作,菜单 Cmd+, 已是最短路径。
