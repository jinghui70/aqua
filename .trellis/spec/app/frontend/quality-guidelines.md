# Quality Guidelines

> Code quality standards for frontend development.

---

## Overview

aqua 前端遵循 Vue 3 组合式 API + TypeScript strict 模式 + element-plus 组件库规范。

---

## Forbidden Patterns

### ❌ 选项式 API

```vue
<!-- ❌ Options API -->
<script>
export default {
  data() {
    return { count: 0 }
  },
  methods: {
    increment() { this.count++ }
  }
}
</script>
```

**✅ 组合式 API**:

```vue
<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
const increment = () => count.value++
</script>
```

### ❌ any 类型

```typescript
// ❌
const data: any = await invoke("project_open", { path })

// ✅
import type { Project } from '@/types/schema'
const data: Project = await invoke<Project>("project_open", { path })
```

### ❌ 直接调 invoke 不做错误处理

```typescript
// ❌ 未处理错误
const project = await invoke("project_open", { path })

// ✅ 统一错误处理
try {
  const project = await invoke<Project>("project_open", { path })
  // ...
} catch (err) {
  ElMessage.error(`打开失败: ${err}`)
}
```

---

## Required Patterns

### ✅ `<script setup lang="ts">`

所有 .vue 文件必须用 `<script setup lang="ts">`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const msg = ref<string>('加载中...')
</script>
```

### ✅ Tauri invoke 封装在 composables

```typescript
// composables/useTauri.ts
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

export function useTauri() {
  async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await tauriInvoke<T>(cmd, args)
    } catch (err) {
      ElMessage.error(`操作失败: ${err}`)
      throw err
    }
  }
  
  return { invoke }
}
```

### ✅ 类型定义对齐 Rust

```typescript
// types/schema.ts - 与 aqua-core Rust 类型一致
export interface Project {
  version: string
  basePackage: string
  bizTypes: BizTypeDefine[]
  enums: EnumDefine[]
  groups: GroupDefine[]
  tables: Table[]
}

export interface Table {
  code: string
  name: string
  group: string
  fields: Field[]
  indexes?: Index[]
  comment?: string
}
```

字段名对齐 JSON(驼峰),与 Rust `#[serde(rename)]` 后的输出一致。

---

## Testing Requirements

**当前**: 无测试(骨架阶段,手动验证即可)。

**后续**: 主流程手动测试(打开/保存/生成/导入),复杂组件加 Vitest 单测。

---

## Code Review Checklist

- [ ] 用 `<script setup lang="ts">`,不用 Options API
- [ ] 无 `any` 类型,所有 invoke 返回值有明确类型
- [ ] Tauri invoke 统一错误处理(try-catch + ElMessage)
- [ ] 类型定义对齐 Rust(字段名驼峰)
- [ ] 组件命名多词(TableTree / FieldEditor)
- [ ] composables 用 `use` 前缀

---

## Linting

- **TypeScript**: `strict: true` + `noEmit: true`(类型检查)
- **Vue**: 默认 ESLint 规则(Vite 模板自带)
- **格式化**: Prettier(按需,当前未配置)

---

## Examples

**当前状态**: `app/src/App.vue` - 占位组件。

**目标模式**:
- 组合式 API: 所有 .vue 用 `<script setup>`
- TS 严格: 所有 API 调用带类型
- 错误处理: composables 统一封装
