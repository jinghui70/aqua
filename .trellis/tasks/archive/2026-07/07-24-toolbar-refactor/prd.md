# 主工作界面工具栏重构

## Goal

重构主工作界面顶部工具栏(`app/src/layout/AppToolbar.vue`):补齐高频"保存"入口、图标化视觉改造、放大按钮点击区让比例协调好看、只读锁移到最右。核心痛点——**当前按钮太扁不好点**,且最高频的保存操作只能点 macOS 顶部菜单栏。

## Background

- 主界面 = `AppLayout.vue`(有项目时):`AppToolbar`(顶部) + 左分组树 + 多标签工作区 + `StatusBar`(底部)。
- 工具栏现状(`AppToolbar.vue:12-48`):`导入` · `导出`(DDL/diff/字符串变量下拉) │ `只读锁` │ `配置` · `数据集` · `驱动管理`。全部 `size="small"` + 容器 `py-6`,按钮矮扁难点。
- **文件操作只在原生菜单**(`useMenuActions.ts` handle + `src-tauri/src/lib.rs` build_menu),且菜单项用 `.text()` **无 accelerator**——连 Cmd+S 都没有,保存唯一入口是 macOS 顶部菜单栏。
- 工具栏动作经 `useUiStore`(弹窗开关) / `router.push` / `store` action 触发。保存链路:`useMenuActions.doSave()` → `store.saveProject()`。
- `store.dirty`(project.ts:38) 已存在并导出,保存后置 false,`saveProject`/`confirmIfDirty` 可复用。
- `store.readOnly` + `store.toggleReadOnly` 已存在(只读锁用)。
- 图标现状:**未装任何图标库**,现有"图标"全是 emoji(📁📄🔒🔓←)。Welcome.vue 已有成熟的"图标上/文字下竖排"视觉语言(`flex flex-col items-center` + `text-32` 图标 + `text-14` 文字)。
- 样式:项目用 **unocss**(`uno.config.ts:1-2` 仅启 `presetUno` + `presetRemToPx`,**未启 `presetIcons`**);记忆约定用原子类、不写 scoped CSS。

## Requirements

### R1. 补齐"保存"到工具栏(仅保存,高频)
- 只加**保存**按钮;新建/打开/另存/最近/关闭 仍走原生菜单(不搬)。
- **只读态(`store.readOnly`)时不显示**保存按钮(只读不能编辑,保存无意义)。
- 非只读时:按 `store.dirty` 联动——无改动时 `disabled`;dirty 时可点 + **红点提示**未保存。
- 点击走既有链路(`store.saveProject(store.currentPath)`,无路径时走保存对话框,复用 `useMenuActions.doSave` 逻辑)。
- 加 **Cmd/Ctrl+S 快捷键**:走 Rust 菜单 accelerator 方案——`lib.rs` 的 `file.save` 由 `.text(...)` 改为带 `.accelerator("CmdOrCtrl+S")`,复用现有 `menu` event → `useMenuActions` 分发链路(零新前端基础设施;`anyDialogOpen` 时按既有逻辑忽略)。

### R2. 图标方案 = unocss presetIcons
- `uno.config.ts` 加 `presetIcons`,装 `@iconify-json/mdi`(或等价图标集)作 devDependency。
- 图标以 class 使用(如 `i-mdi-content-save`),不引入 `@element-plus/icons-vue`,不新增运行时依赖。
- 工具栏所有按钮(保存/导入/导出/配置/数据集/驱动/只读锁)换用矢量图标替代 emoji 与纯文字。

### R3. 按钮形态 = 比例协调、点击区放大
- 采用**图标 + 文字**(可"上图标下文字"竖排,参照 Welcome 卡片视觉语言),按钮比例协调、不扁、易点。
- 调整工具栏容器高度与按钮 padding/尺寸,消除"太扁"问题。
- 主操作(保存/导入/导出)与管理操作(配置/数据集/驱动)按功能域分组,divider 分隔。

### R4. 只读锁移到最右
- 只读锁按钮从中部移到工具栏**最右端**(`flex-1` spacer 顶开),与左侧功能按钮拉开。
- 文字与图标优化(替代 `🔒 解锁编辑`/`🔓 加锁` emoji)。

## Acceptance Criteria

- [ ] 非只读 + dirty:工具栏显示保存按钮,可点,带红点;点击后保存成功、红点消失、按钮转 disabled。
- [ ] 非只读 + 无改动:保存按钮可见但 disabled、无红点。
- [ ] 只读态:保存按钮不显示。
- [ ] Cmd/Ctrl+S 触发保存(与点按钮等效);macOS 菜单"保存"项显示 ⌘S。
- [ ] 工具栏按钮均为矢量图标(presetIcons)+ 文字,无 emoji;按钮比例协调、点击区明显大于改造前。
- [ ] 只读锁按钮位于工具栏最右端。
- [ ] `pnpm vue-tsc` 通过;`presetIcons` 图标正常渲染(构建产物含对应 SVG)。
- [ ] 现有功能不回归:导入/导出/配置/数据集/驱动/只读切换 全部照常触发。

## Out of Scope

- 搬移 新建/打开/另存/最近/关闭 到工具栏(仍走原生菜单)。
- 窄窗响应式折叠(按钮溢出换行本次不处理)。
- 替换 GroupTreeAside/DatasetManage/Welcome 等**非工具栏**处的 emoji(presetIcons 装好后可后续统一,本次不做)。
- StatusBar / 分组树 / TabWorkspace 的改造。

## Technical Notes

- 快捷键选 Rust accelerator 而非前端 keydown:复用现有 `menu` event 链路、零新监听基础设施、原生菜单可显示 ⌘S、符合桌面惯例。前端当前无任何 keydown 监听。
- presetIcons 选 mdi(Material Design Icons):图标全、命名直观(`i-mdi-content-save`/`i-mdi-import`/`i-mdi-export`/`i-mdi-cog`/`i-mdi-database`/`i-mdi-lock`/`i-mdi-lock-open`)。
- 保存按钮红点可用 unocss 定位类做小圆点(`absolute` + `rounded-full` + `bg-red-500`),或 el-badge is-dot。
