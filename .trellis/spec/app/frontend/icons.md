# Icons

> How icons are used in this project.

---

## Overview

aqua 前端图标用 **unocss `presetIcons`**(iconify mdi 集),配置在 `app/uno.config.ts`。构建期内联 SVG,无运行时依赖。**不用** `@element-plus/icons-vue`(项目未装该依赖)。

---

## Usage

```vue
<el-button>
  <span class="i-mdi-content-save w-16 h-16 mr-4" />
  保存
</el-button>
```

- 类名格式 `i-{collection}-{icon}`(本项目统一用 **mdi** 集)。
- 尺寸靠 `w-/h-` 显式给(`presetRemToPx` 下 `1em=4px`,不显式给会太小;常用 `w-16 h-16`)。
- `presetIcons.extraProperties` 已设 `vertical-align:middle`,自动与文字基线对齐。
- 图标与文字间距用 `mr-4`。
- 放 `el-button` 默认 slot(`:icon` prop 不适用,那是给 element-plus 图标组件用的)。

---

## Icon Library

查 https://icones.js.org/ 的 **mdi** 集(Material Design Icons)。常用:

| 场景 | 类名 |
|---|---|
| 保存 | `i-mdi-content-save` |
| 拷贝 | `i-mdi-content-copy` |
| 粘贴 | `i-mdi-content-paste` |
| 删除 | `i-mdi-delete` |
| 设置 | `i-mdi-cog` |
| 表 | `i-mdi-table` |
| 数据库 | `i-mdi-database` |

---

## Common Mistakes

### ❌ 用 @element-plus/icons-vue

```vue
import { Delete } from "@element-plus/icons-vue"  // ❌ 项目未装
<el-button :icon="Delete">删除</el-button>
```

**✅ 正确**: unocss 图标 `<span class="i-mdi-..." />`。

### ❌ 不给尺寸

```vue
<span class="i-mdi-delete" />  <!-- ❌ 1em=4px,图标太小 -->
```

**✅ 正确**: 显式 `w-16 h-16`。

---

## Examples

- `app/src/layout/AppToolbar.vue` - 工具栏按钮(图标 + 文字 + 角标)
- `app/src/views/table-editor/FieldsTab.vue` - 拷贝/粘贴/删除按钮
