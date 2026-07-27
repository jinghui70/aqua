# Implement: 字段拷贝与粘贴

## 实现清单

1. **新建 `app/src/stores/clipboard.ts`**:`useClipboardStore`(setup store + `acceptHMRUpdate`),`Field[]` + `set`(深拷贝入)/`get`(深拷贝出)/`clear`/`has` computed。
2. **`app/src/views/table-editor/FieldsTab.vue`**:
   a. 表格加 `<el-table-column type="selection" width="36">`;readOnly 时 `:selectable` 返回 false。
   b. `@selection-change` 记录选中(row 对象引用数组)。
   c. 顶部工具栏加"拷贝""粘贴""删除"按钮;可用性 computed(选中数>0 / 剪贴板 `has` / `!readOnly`)。
   d. `copySelected`:深拷贝选中 -> `clipboard.set`;`ElMessage.success("已拷贝 N 个字段")`。
   e. `paste`:`clipboard.get` -> 原子冲突检查(目标表 code Set + 剪贴板内部 code 去重) -> 全过则 `push` 末尾;冲突则 `ElMessage.warning` 提示冲突 code 列表,不执行。
   f. `deleteSelected`:选中按 idx 降序 `splice` + `store.removeFieldFromIndexes(tableId, code)`;`ElMessage.success("已删除 N 个字段")`。
   g. 删除 `copyField` 函数 + 操作列"拷贝""删"按钮(保留"详情")。
3. **验证**:`cd app && npx vue-tsc --noEmit`。

## 手动验证

- 多选字段 -> 拷贝 -> 切表 -> 粘贴 -> 字段出现在末尾。
- paste 时 code 冲突(含剪贴板内部重复) -> 提示冲突 code,不粘贴。
- 删除选中 -> 索引引用清理(`removeFieldFromIndexes`)。
- 切项目 -> 剪贴板保留 -> 跨项目 paste -> bizType 降级显示 code 原值。
- readOnly -> 选择列、"拷贝"可见可用;"粘贴""删除"隐藏。

## 风险点

- FieldsTab 现有直接改 `props.fields`(非 emit),copy/paste/delete 沿用。
- 删除按 idx 降序 splice,避免 index 偏移。
- selection 用 row 对象引用,splice 后按引用定位 idx。
