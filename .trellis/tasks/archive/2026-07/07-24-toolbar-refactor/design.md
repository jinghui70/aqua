# Design - 主工作界面工具栏重构

## 影响面

| 层 | 文件 | 改动 |
|---|---|---|
| 构建/样式 | `app/uno.config.ts` | 加 `presetIcons()` |
| 依赖 | `app/package.json` | devDep 加 `@iconify-json/mdi` + `@unocss/preset-icons`(若主包未内置) |
| 前端 | `app/src/layout/AppToolbar.vue` | 重构:加保存按钮、图标化、分组、只读锁移右 |
| Rust | `src-tauri/src/lib.rs` | `file.save` 菜单项加 `.accelerator("CmdOrCtrl+S")` |

> 保存动作链路、`store.dirty`/`readOnly`/`saveProject` 均已存在,**不新增 store/composable**。

## 1. presetIcons 引入(一次性基础设施)

- unocss 主包已装(`unocss@66`),`presetIcons` 从 `unocss` 导出;图标数据按需从 `@iconify-json/mdi` 读取(纯 devDep,构建期内联为 SVG,无运行时依赖)。**用在线图标集,不用本地 ./icons(FileSystemIconLoader)**。
- `uno.config.ts`(**增量加 presetIcons,不改既有 presetUno/presetRemToPx/rules 结构**):
  ```ts
  import { defineConfig, presetUno, presetIcons } from "unocss";
  // presets: [
  //   presetUno(),
  //   presetRemToPx({ baseFontSize: 4 }),
  //   presetIcons({ extraProperties: { display: "inline-block", "vertical-align": "middle" } }),
  // ]
  ```
- `extraProperties` 采纳用户常用配置:`inline-block` + `vertical-align: middle` → 图标与按钮文字基线对齐,避免错位。
- 验证:`i-mdi-content-save` 类能渲染出 SVG(dev + build 均测)。
- 风险:presetRemToPx 的 baseFontSize:4 会影响图标默认尺寸(1em=4px)。图标尺寸显式用 `w-18 h-18` 类控制,不依赖默认 em。
- **范围外**(用户参考配置里有,但本次不引入):`presetWind3` 替换 `presetUno`、`theme` 绑 element-plus CSS 变量、`transformerVariantGroup`、圆角/字号/色板体系——属整套设计系统对齐,另开任务。

## 2. AppToolbar 重构

### 布局(左 → 右,flex + 分组 divider)
```
[💾 保存]  │  [导入] [导出▾]  │  [配置] [数据集] [驱动]  ←flex-1→  [🔒 只读锁]
  文件组          生成组              管理组                        (最右)
```

### 保存按钮(仅非只读显示)
- `v-if="!store.readOnly"`。
- `:disabled="!store.dirty"`。
- dirty 红点:按钮内 `<span>` 定位小圆点(`absolute -top-2 -right-2 w-6 h-6 rounded-full bg-red-500`),`v-if="store.dirty"`。
- `@click` 调用保存逻辑。**复用问题**:`useMenuActions.doSave()` 现是 composable 内私有函数,工具栏点击也要走它 → 抽取或在工具栏内联同一逻辑(见下"保存逻辑归属")。

### 保存逻辑归属(决策)
- 现状:`doSave()` 私有于 `useMenuActions`。工具栏按钮 + Cmd+S(经菜单)若各写一份易分叉。
- 方案:Cmd+S 走菜单 event → `useMenuActions.doSave`(已有);工具栏按钮点击**也发同一 menu 逻辑** → 最简做法是工具栏直接 `store.saveProject(store.currentPath)` + 无路径兜底。但"无路径走保存对话框"逻辑在 `doSave` 里。
- **定**:把 `doSave` 逻辑保留在 `useMenuActions`,并从 `useMenuActions()` 返回 `doSave`,供 AppToolbar import 调用。单一实现,双入口(按钮 + Cmd+S)共用。

### 按钮形态(R3)
- 图标+文字,竖排或横排取协调比例;容器高度从 `py-6` 增大(如 `py-8`~`py-10`),按钮 `size` 视觉放大点击区。
- 图标类:保存 `i-mdi-content-save`、导入 `i-mdi-database-import`、导出 `i-mdi-export`、配置 `i-mdi-cog`、数据集 `i-mdi-table`、驱动 `i-mdi-database-cog`、只读锁 `i-mdi-lock`/`i-mdi-lock-open-variant`。

## 3. Rust 菜单 accelerator(API 已核实 @ tauri 2.11.5)

- `SubmenuBuilder.text(id, label)` 无 accelerator 变体;但 `SubmenuBuilder` 有 `.item(&dyn IsMenuItem)`(builders/menu.rs:222),`MenuItemBuilder::new(label).id(id).accelerator(...).build(app)?` 产出的 `MenuItem` 实现 `IsMenuItem`(builders/normal.rs:20/55/61)。
- 改法:`file.save` 从 `.text("file.save","保存")` 抽出,单独构造带快捷键的 MenuItem 再 `.item()` 插入:
  ```rust
  use tauri::menu::MenuItemBuilder;
  let save = MenuItemBuilder::new("保存").id("file.save").accelerator("CmdOrCtrl+S").build(app)?;
  let file = SubmenuBuilder::new(app, "文件")
      .text("file.new", "新建项目")
      .text("file.open", "打开项目")
      .text("file.recent", "最近项目")
      .item(&save)                 // ← 带 ⌘S
      .text("file.saveAs", "另存为")
      .text("file.close", "关闭项目");
  ```
- 触发后 emit `menu`("file.save") → `useMenuActions.handle` → `doSave`,链路不变。
- `anyDialogOpen` 时忽略(既有逻辑),避免弹窗中误存。

## 兼容 / 回归

- 导入/导出/配置/数据集/驱动/只读切换:仅换图标+重排,`@click` 目标不变。
- 原生菜单文件操作全保留,仅 save 加快捷键。
- 无数据结构 / 存储格式变更。

## 验证

- `pnpm vue-tsc`(前端类型)。
- `pnpm dev` 手测:presetIcons 渲染、保存按钮 dirty/readOnly 三态、红点、Cmd+S、菜单显 ⌘S、各按钮触发。
- `cargo build`(Rust 菜单编译)。
