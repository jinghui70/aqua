# Implement - 主工作界面工具栏重构

## 执行顺序

### 1. presetIcons 基础设施
- [ ] `cd app && pnpm add -D @iconify-json/mdi`(在线图标数据集;presetIcons 从 unocss 主包导出)。
- [ ] `app/uno.config.ts`:import `presetIcons`,增量加入 presets 数组(不改既有 presetUno/presetRemToPx/rules):`presetIcons({ extraProperties: { display: "inline-block", "vertical-align": "middle" } })`。
- [ ] 冒烟:在 AppToolbar 临时放一个 `<div class="i-mdi-content-save w-18 h-18" />`,`pnpm dev` 确认 SVG 渲染;确认 presetRemToPx 不破坏图标尺寸(必要时显式 `w-18 h-18`)。

### 2. Rust 菜单加 Cmd+S
- [ ] `src-tauri/src/lib.rs`:import `tauri::menu::MenuItemBuilder`;`file.save` 抽为带 `.accelerator("CmdOrCtrl+S")` 的 MenuItem,用 `.item(&save)` 插入 SubmenuBuilder(见 design §3)。
- [ ] `cargo build`(或 `pnpm tauri dev`)确认编译 + 菜单"保存"显示 ⌘S。

### 3. useMenuActions 暴露 doSave
- [ ] `app/src/composables/useMenuActions.ts`:`return { mount, unmount, handle, doSave }`(doSave 已存在,仅加入返回)。
- [ ] 确认调用处(AppLayout 或 setup)获取方式不受影响。

### 4. AppToolbar 重构(核心)
- [ ] 引 `store`(dirty/readOnly/saveProject/currentProject)、`useMenuActions().doSave`。
- [ ] 布局:文件组[保存] │ 生成组[导入][导出▾] │ 管理组[配置][数据集][驱动] ─flex-1─ [只读锁]。
- [ ] 保存按钮:`v-if="!store.readOnly"` + `:disabled="!store.dirty"` + dirty 红点(`v-if="store.dirty"` 定位小圆点)+ `@click="doSave()"`。
- [ ] 全部按钮换 mdi 图标 + 文字,放大点击区(容器高度 ↑、按钮 padding ↑);只读锁移最右并换图标。
- [ ] 保留所有 `@click` 目标(ui.openImport/openExport/openDatabaseConfig、router.push、toggleReadOnly)。

### 5. 验证
- [ ] `cd app && pnpm vue-tsc --noEmit`。
- [ ] `pnpm dev` 手测清单(= PRD Acceptance Criteria):保存三态(readOnly 隐藏 / 非 dirty disabled / dirty 可点+红点)、Cmd+S、菜单 ⌘S、各按钮触发、图标渲染、只读锁在最右。
- [ ] `cargo build` 通过。

## 风险 / 回滚点

- **presetRemToPx × presetIcons 尺寸冲突**:baseFontSize:4 使 1em=4px,图标若靠 em 会极小。对策:图标显式 `w-18 h-18`/`text-18`。若渲染异常,先在冒烟步骤(1.3)暴露。
- **tauri accelerator API**:已核实 2.11.5 用 MenuItemBuilder(design §3)。若 build 报错,回退——save 项保持 `.text()`,快捷键改前端 `window.addEventListener('keydown')` 兜底(需处理 input 焦点 + preventDefault)。
- **doSave 双入口分叉**:统一走 useMenuActions.doSave,不在工具栏另写保存逻辑。
- 每步独立可回滚;presetIcons(步1)是唯一跨文件基础设施,先做并冒烟。

## 验证命令
```bash
cd app && pnpm vue-tsc --noEmit && pnpm dev   # 前端
cargo build                                    # Rust 菜单
```
