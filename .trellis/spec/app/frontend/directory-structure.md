# Directory Structure

> How frontend code is organized.

---

## Overview

`app/` 是 **Vue 3 + TypeScript + element-plus** 前端,通过 Tauri invoke 与 Rust 后端通信。

**当前状态**: 骨架阶段,仅 `App.vue` + `main.ts` 验证链路。后续扩展遵循 Vue 3 组合式 API + 功能模块分组。

---

## Directory Layout

```
app/
├── src/
│   ├── main.ts              # 入口:挂载 Vue + element-plus
│   ├── App.vue              # 根组件
│   ├── components/          # 通用组件(待建)
│   │   ├── ui/              # 基础 UI(按钮/输入框/对话框封装)
│   │   └── business/        # 业务组件(表字段编辑器/DDL 预览)
│   ├── views/               # 页面级组件(待建)
│   │   ├── ProjectEditor.vue    # schema 编辑主界面
│   │   ├── GeneratorPanel.vue   # DDL/Java 生成配置
│   │   └── ImportWizard.vue     # 从数据库导入向导
│   ├── composables/         # 组合式函数(待建)
│   │   ├── useProject.ts    # 项目状态管理
│   │   └── useTauri.ts      # Tauri invoke 封装
│   ├── types/               # TS 类型定义(待建)
│   │   ├── schema.ts        # Project/Table/Field 类型(对齐 Rust)
│   │   └── api.ts           # Tauri command 参数/返回值类型
│   ├── utils/               # 工具函数(待建)
│   │   └── validation.ts    # 前端校验辅助
│   └── assets/              # 静态资源(待建)
│       ├── styles/          # 全局样式
│       └── icons/           # 图标
├── public/                  # 静态文件(不经 Vite 处理)
├── index.html               # HTML 入口
├── vite.config.ts           # Vite 配置
├── tsconfig.json            # TS 配置
└── package.json             # 依赖:vue/element-plus/tauri-api
```

**组织原则**:
- **功能优先**: views/ 按页面分,不按技术层(不建 pages/list + pages/detail)
- **组件分层**: ui/ 纯展示,business/ 含业务逻辑
- **composables/ 复用逻辑**: 状态管理/API 调用/表单校验,不放 components/

---

## Module Organization

### 当前(骨架)

```
src/
├── App.vue     # 占位:调 greet command 验证 Tauri 链路
└── main.ts     # 挂载 Vue + element-plus
```

### 目标结构(待扩展)

```
src/
├── views/
│   └── ProjectEditor.vue       # 主编辑界面:左侧树(表/字段)+右侧属性面板
├── components/
│   ├── ui/
│   │   └── Dialog.vue          # element-plus el-dialog 封装
│   └── business/
│       ├── TableTree.vue       # 表树组件
│       └── FieldEditor.vue     # 字段编辑表单
├── composables/
│   ├── useProject.ts           # ref<Project>, openProject/saveProject
│   └── useTauri.ts             # invoke 封装 + 错误处理
└── types/
    └── schema.ts               # Project/Table/Field 接口(对齐 Rust)
```

---

## Naming Conventions

- **文件名**: PascalCase.vue(组件) / camelCase.ts(工具/类型)
- **组件**: 多词组合(TableTree / FieldEditor,避免单词 Table / Field)
- **composables**: `use` 前缀(useProject / useTauri)
- **类型文件**: 复数或描述性(schema.ts / api.ts,不叫 types.ts)

---

## Import Paths

用 Vite `@/` 别名指向 `src/`:

```typescript
// vite.config.ts
export default defineConfig({
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
})
```

**导入示例**:

```typescript
// ✅ 绝对路径
import { useProject } from '@/composables/useProject'
import type { Project } from '@/types/schema'

// ❌ 相对路径(深层嵌套时难维护)
import { useProject } from '../../../composables/useProject'
```

---

## Examples

**当前状态**: `app/src/App.vue` - 占位组件,调 `invoke("greet")` 验证 Tauri 链路通。

**目标参考**(待移植):
- `~/work/aqua-legacy/packages/web/src/` - 旧 TS 版前端逻辑蓝本
- element-plus 官方示例: 表单/树/对话框组件模式
