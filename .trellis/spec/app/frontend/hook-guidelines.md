# Hook Guidelines (Composables)

> How composables are used in Vue 3.

---

## Overview

Vue 3 用 **composables** 代替 React hooks。命名 `use*`,返回响应式状态 + 操作函数。

---

## Custom Composable Patterns

### 基础模式

```typescript
// composables/useCounter.ts
import { ref } from 'vue'

export function useCounter(initialValue = 0) {
  const count = ref(initialValue)
  
  const increment = () => count.value++
  const decrement = () => count.value--
  const reset = () => count.value = initialValue
  
  return {
    count,
    increment,
    decrement,
    reset,
  }
}
```

**使用**:

```vue
<script setup lang="ts">
import { useCounter } from '@/composables/useCounter'

const { count, increment } = useCounter(10)
</script>
```

### 全局状态 composable

```typescript
// composables/useProject.ts
import { ref } from 'vue'
import type { Project } from '@/types/schema'

// 模块级 ref,所有调用者共享
const currentProject = ref<Project | null>(null)

export function useProject() {
  // 返回同一个 currentProject
  return {
    currentProject,
    openProject,
    saveProject,
  }
}
```

---

## Data Fetching (Tauri invoke)

```typescript
// composables/useTauri.ts
import { ref } from 'vue'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

export function useTauri() {
  const loading = ref(false)
  
  async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    loading.value = true
    try {
      const result = await tauriInvoke<T>(cmd, args)
      return result
    } catch (err) {
      ElMessage.error(`操作失败: ${err}`)
      throw err
    } finally {
      loading.value = false
    }
  }
  
  return { invoke, loading }
}
```

---

## Naming Conventions

- **函数名**: `use` 前缀 + PascalCase(useProject / useTauri)
- **文件名**: camelCase.ts(useProject.ts)
- **返回值**: 对象解构(不返回数组)

---

## Common Mistakes

### ❌ 在 composable 外调用

```typescript
// ❌ 全局调用,不在 setup 里
const { count } = useCounter()

export default {
  setup() {
    // ...
  }
}
```

**✅ 正确**: 只在 `setup` / `<script setup>` 里调用。

```vue
<script setup lang="ts">
const { count } = useCounter()  // ✅
</script>
```

---

## Examples

- `useProject()` - 管理当前项目
- `useTauri()` - 封装 Tauri invoke
