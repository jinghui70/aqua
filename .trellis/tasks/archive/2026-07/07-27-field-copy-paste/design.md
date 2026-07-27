# Design: 字段拷贝与粘贴

## 架构与边界

- **剪贴板**:独立 Pinia store `useClipboardStore`(`app/src/stores/clipboard.ts`),存 `Field[]`。不随 project 切换清空(跨项目支持 D7)。
- **FieldsTab**:加多选 + 顶部按钮,直接操作 `props.fields`(`table.fields` 引用,Pinia 响应式),遵循现有 `addField`/`removeField` 模式。
- **操作列精简**:去掉"拷贝""删",留"详情"。

## 数据流

- **copy**:选中 fields -> 深拷贝(`JSON.parse(JSON.stringify)`) -> `clipboard.set(fields)`。
- **paste**:`clipboard.get()`(深拷贝出) -> 原子冲突检查(目标表 code Set + 剪贴板内部 code 去重) -> 全通过则 `push` 到 `props.fields` 末尾;任一冲突则 `ElMessage.warning` 提示冲突 code,不执行。
- **delete**:选中 fields -> 按 idx 降序 `splice(props.fields)` + `store.removeFieldFromIndexes(tableId, code)` 级联清理索引。

## 契约:useClipboardStore

```ts
fields: Ref<Field[]>           // 剪贴板内容
has: ComputedRef<boolean>      // 非空判断(粘贴按钮可用性)
set(fields: Field[]): void     // 深拷贝入
get(): Field[]                 // 深拷贝出(避免引用共享)
clear(): void
```

## 兼容性

- 现有 `copyField`(原地拷贝插入副本)废弃删除。
- 操作列"拷贝""删"按钮删除,留"详情"。
- 删除级联索引复用现有 `removeFieldFromIndexes`,行为不变。

## 权衡

- **原子冲突 vs 部分粘贴**:原子更安全(避免部分粘贴后 code 语义混乱),用户明确要求(D5)。
- **剪贴板独立 vs 随项目清空**:独立支持跨项目(D7),接受 bizType 降级(数据照搬,UI fallback 显示 code 原值)。
- **末尾插入 vs 指定位置**:末尾最简,靠现有拖拽调序(D4)。

## 风险

- **bizType 跨项目降级**:仅显示层(现有 `bizTypeLabel` fallback 显示 code 原值),数据照搬。用户需手动校正 bizType。
- **深拷贝**:JSON 方式安全(Field 纯数据,无函数/Date/循环引用)。
- **删除 index 偏移**:按 idx 降序 splice 规避。
- **selection 引用**:el-table `@selection-change` 返回 row 对象数组,splice 后按对象引用定位 idx(不依赖选中时的 idx)。
