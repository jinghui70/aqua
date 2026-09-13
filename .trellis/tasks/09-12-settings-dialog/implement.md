# 执行计划:配置中心改为 Cmd+, 设置对话框

## 顺序清单

前端为主,Rust 菜单独立可并行;最后统一清理。

### 1. ui store(`app/src/stores/ui.ts`)
- [ ] 加 `settingsVisible` / `settingsPanel` / `openSettings(panel?)`,导出 `SettingsPanel` 类型
- [ ] `anyDialogOpen` 纳入 `settingsVisible`
- [ ] 删 `databaseConfigVisible` / `openDatabaseConfig`

### 2. Rust 菜单(`src-tauri/src/lib.rs`)
- [ ] macOS 应用菜单 `.preferences()`(验证实际 emit id,必要时在 on_menu_event 映射为 `app.settings`)
- [ ] Windows/Linux 文件菜单加"设置"项(MenuItemBuilder + `CmdOrCtrl+Comma`)
- [ ] `cargo check`

### 3. 前端组件
- [ ] `views/config/DriverPanel.vue`:从 DatabaseConfigDialog 抽表格主体
- [ ] `components/SettingsDialog.vue`:el-dialog 900px + 左导航五分类 + 右面板 v-if 链;`:deep(.el-dialog__body)` 归零 padding;activePanel 从 ui.settingsPanel 初始化
- [ ] 挂载 SettingsDialog(AppLayout 或 App.vue,对齐现有 Dialog 挂载位置——查 DatabaseConfigDialog 挂哪就挂哪)

### 4. 入口与清理
- [ ] `useMenuActions.ts` 加 `app.settings` 分支
- [ ] `AppToolbar.vue`:配置按钮 → `ui.openSettings()`;删驱动管理按钮
- [ ] `router/index.ts` 删 `/config` 路由
- [ ] 删 `views/ConfigCenter.vue`、`components/DatabaseConfigDialog.vue`
- [ ] grep 残余引用:`/config`、`openDatabaseConfig`、`databaseConfigVisible`、`ConfigCenter`

### 5. 验证
- [ ] `cd app && npx vue-tsc --noEmit`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] 手工(macOS,`pnpm dev`):Cmd+, 打开;五分类切换;驱动安装/卸载;无项目时打开不报错;对话框开时 Cmd+S 不触发保存
- [ ] Windows 验证留给发版(GitHub Actions 打包后)

## 验证命令

```bash
cd app && npx vue-tsc --noEmit
cargo check --manifest-path src-tauri/Cargo.toml
pnpm dev   # 手工验证 AC1/AC3/AC4/AC5
```

## 风险点与回滚

- `.preferences()` 的实际 emit id 以实测为准(design §2 已写明兜底方案:Rust 侧映射)。
- `CmdOrCtrl+Comma` accelerator 语法若不被接受,退回 `CmdOrCtrl+,`(Tauri 2 两者都应支持,实测为准)。
- Panel 组件从路由容器搬进 dialog:布局假设(overflow-auto 父容器)需逐一目检,尤其 DataSourcePanel 的高度撑满逻辑(`h-full`)在 dialog body 里的表现。
- 回滚:本改动无数据迁移,git revert 即可。
