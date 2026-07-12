# Enum 管理页

## 背景
parent frontend-rework child。fe-arch 已建 EnumManage 占位路由页。菜单"配置>枚举管理"打开。

## 目标(design.md §6.6)
全局枚举列表 + 编辑。字段引用全局枚举或内联(内联在表编辑器,本任务只做全局枚举管理)。

## 包含
- 左列表: 全局枚举(EnumDefine)+ 新建/删除
- 右编辑: code/name/package/hasCode + values 表格
- values 表格: id/name/code?(hasCode=true 必填)/color? 增删
- color 用预置 13 色下拉(§3.5)
- 直改 store.currentProject.enums

## 范围(不含)
- 字段引用枚举的选择(表编辑器 enum 字段,后续增强)
- 内联枚举编辑(表编辑器内)

## 验收标准
- [ ] EnumManage.vue: 左列表 + 右编辑
- [ ] enum CRUD
- [ ] values 子表增删(id/name/code/color)
- [ ] hasCode 开关 + color 预置下拉
- [ ] 直改 store(响应式)
- [ ] pnpm build 通过

## 约束
- code 唯一, ElMessageBox, unocss px + element-plus
- color 预置: success/error/warning/info/primary/danger/red/orange/yellow/green/blue/purple/grey
