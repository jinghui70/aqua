# Component Guidelines

> How components are built in Vue 3.

---

## Overview

aqua 前端用 **Vue 3 组合式 API + element-plus** 组件库。组件分两层:
- **ui/**: 通用 UI 组件(对 element-plus 的薄封装)
- **business/**: 业务组件(表编辑器/字段面板)

---

## Component Structure

```vue
<!-- components/business/TableTree.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import type { Table } from '@/types/schema'

// Props
interface Props {
  tables: Table[]
  selectedId?: string
}
const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'select', table: Table): void
}
const emit = defineEmits<Emits>()

// Local state
const expandedKeys = ref<string[]>([])

// Methods
function handleNodeClick(table: Table) {
  emit('select', table)
}
</script>

<template>
  <el-tree
    :data="tables"
    :expanded-keys="expandedKeys"
    @node-click="handleNodeClick"
  />
</template>

<style scoped>
/* 组件私有样式 */
</style>
```

---

## Props Conventions

### ✅ TypeScript interface 定义 Props

```typescript
interface Props {
  table: Table
  readonly?: boolean  // 可选 prop
}
const props = defineProps<Props>()
```

### ✅ withDefaults 设置默认值

```typescript
interface Props {
  initialValue?: number
}
const props = withDefaults(defineProps<Props>(), {
  initialValue: 0
})
```

---

## Emits Conventions

```typescript
interface Emits {
  (e: 'update:modelValue', value: string): void
  (e: 'submit', data: FormData): void
}
const emit = defineEmits<Emits>()

// 使用
emit('update:modelValue', newValue)
emit('submit', formData)
```

---

## Styling Patterns

### Scoped styles

```vue
<style scoped>
.table-tree {
  height: 100%;
}
</style>
```

**原则**: 
- 所有组件用 `<style scoped>`
- 不用 CSS Modules / Tailwind(element-plus 已提供样式)
- 全局样式放 `assets/styles/global.css`

---

## Accessibility

element-plus 已内置 ARIA 属性,自定义组件遵循:
- 按钮用 `<el-button>`,不用 `<div @click>`
- 表单用 `<el-form>`,自动关联 label
- 对话框用 `<el-dialog>`,自动管理焦点

---

## Common Mistakes

### ❌ 直接修改 props

```typescript
const props = defineProps<Props>()
props.table.name = '新名称'  // ❌ props 只读
```

**✅ 正确**: emit 事件给父组件修改。

```typescript
emit('update:table', { ...props.table, name: '新名称' })
```

---

## Examples

**当前状态**: `App.vue` 占位组件。

**目标模式**: 
- ui/ 封装 element-plus
- business/ 业务组件
- `<script setup>` + TS interface Props/Emits
