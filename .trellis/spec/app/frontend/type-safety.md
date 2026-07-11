# Type Safety

> Type safety patterns in Vue 3 + TypeScript.

---

## Overview

aqua 前端用 **TypeScript strict 模式**,所有类型对齐 Rust 后端(通过 Tauri invoke 传递 JSON)。

---

## Type Organization

```
src/types/
├── schema.ts       # Project/Table/Field(对齐 Rust aqua_core::schema)
├── api.ts          # Tauri command 参数/返回值类型
└── ui.ts           # 前端专用类型(如树节点/表单数据)
```

### schema.ts (对齐 Rust)

```typescript
// 与 Rust aqua_core::schema::Project 一致
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

**原则**: 
- 字段名驼峰(对齐 Rust `#[serde(rename)]` 后的 JSON)
- 可选字段用 `?:`(对齐 Rust `Option<T>`)

### api.ts (Tauri commands)

```typescript
// Tauri command 签名
export type TauriCommands = {
  project_open: (args: { path: string }) => Promise<Project>
  project_save: (args: { path: string; project: Project }) => Promise<void>
  generate_ddl: (args: { project: Project; dialect: string }) => Promise<string>
}
```

---

## Validation

**当前**: 无运行时校验(Rust 后端已校验,前端信任 Tauri invoke 返回值)。

**后续**: 若需前端校验表单,用 element-plus 内置规则:

```vue
<script setup lang="ts">
import { reactive } from 'vue'

const rules = {
  name: [
    { required: true, message: '请输入表名', trigger: 'blur' },
    { min: 1, max: 64, message: '长度 1-64 字符', trigger: 'blur' }
  ]
}

const form = reactive({ name: '' })
</script>

<template>
  <el-form :model="form" :rules="rules">
    <el-form-item label="表名" prop="name">
      <el-input v-model="form.name" />
    </el-form-item>
  </el-form>
</template>
```

---

## Common Patterns

### Type guards

```typescript
export function isProject(value: unknown): value is Project {
  return (
    typeof value === 'object' &&
    value !== null &&
    'version' in value &&
    'tables' in value
  )
}
```

### Generic composables

```typescript
export function useAsync<T>(fn: () => Promise<T>) {
  const data = ref<T | null>(null)
  const error = ref<Error | null>(null)
  const loading = ref(false)
  
  async function execute() {
    loading.value = true
    try {
      data.value = await fn()
    } catch (e) {
      error.value = e as Error
    } finally {
      loading.value = false
    }
  }
  
  return { data, error, loading, execute }
}
```

---

## Forbidden Patterns

### ❌ any 类型

```typescript
const data: any = await invoke("project_open", { path })  // ❌
```

**✅ 正确**: 明确类型。

```typescript
const data = await invoke<Project>("project_open", { path })
```

### ❌ 类型断言绕过检查

```typescript
const table = data as Table  // ❌ 不确定 data 结构
```

**✅ 正确**: Type guard 或 Rust 校验保证。

```typescript
if (isTable(data)) {
  const table: Table = data
}
```

### ❌ 空对象类型

```typescript
const config: {} = { ... }  // ❌ {} 允许任意属性
```

**✅ 正确**: `Record<string, unknown>` 或具体接口。

```typescript
const config: Record<string, string> = { ... }
```

---

## Examples

**当前状态**: 无类型文件(App.vue 仅调 `invoke<string>`)。

**目标模式**:
- `types/schema.ts` 对齐 Rust
- 所有 invoke 调用带泛型 `invoke<T>`
- strict 模式,无 `any`
