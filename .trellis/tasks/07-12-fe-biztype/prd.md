# bizType 管理页

## 背景
parent frontend-rework child。fe-arch 已建 BizTypeManage 占位路由页。菜单"配置>业务类型管理"打开。

## 目标(design.md §6.5)
左列表 + 右编辑的 bizType 管理页。

## 包含
- 左侧: bizType 列表(内置标记不可删,自定义可删)+ 新建
- 右侧编辑: bizType(code)/name/description/supportedDataTypes(dataType+默认length/precision/scale)/bizTypeData.fields
- supportedDataTypes 表格增删(选 dataType + 默认值)
- bizTypeData.fields 表格增删(name/type(string|number)/description/required)
- 直改 store.currentProject.bizTypes

## 范围(不含)
- 内置 bizType 清单(design.md §11 待定,本期全部视为自定义可删)
- 字段选 bizType 后的表单联动(表编辑器里,后续增强)

## 验收标准
- [ ] BizTypeManage.vue: 左列表 + 右编辑
- [ ] bizType CRUD(新建/删除/编辑)
- [ ] supportedDataTypes 子表格增删
- [ ] bizTypeData.fields 子表格增删
- [ ] 直改 store(Pinia 响应式)
- [ ] 无项目时空状态提示
- [ ] pnpm build 通过

## 约束
- code 唯一校验, ElMessageBox, unocss px + element-plus
