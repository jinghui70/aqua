# 设计

## 组件边界

新增两个聚焦弹窗，改 FieldsTab，删 FieldDetailDialog。

### `BizTypeEditDialog.vue`（新）
- props: `{ modelValue: boolean; field: Field | null }`，emit `update:modelValue`。
- draft 完整副本（`watch(visible)` 深拷贝 field）。
- 模板：bizType 下拉 + dataType 下拉（`showDataTypeSelect`：非 Enum bizType 且 supportedDataTypes.length>1 时显示，选项为该 bizType 支持类型）+ 条件区：
  - `isEnumBizType`：内联枚举配置（枚举名/hasCode/枚举值表 id/name/code/color + 增删）。
  - 否则：`bizTypeData.fields` 动态表单（string->el-input，number->el-input-number，placeholder 显示 default）。
- `onBizTypeChange`：Enum 强制 VARCHAR + 初始化 inlineEnum + 清 bizTypeData；普通 bizType 校正 dataType + applyDefaults + initBizTypeData；清空清 enum。
- `onDataTypeChange`（弹窗内）：cleanupDataType + applyDefaults。
- 保存：`cleanBizTypeData` -> 校验（enum 仅 VARCHAR、hasCode code 必填）-> `Object.assign(field, draft)` -> 关闭。无"已保存"提示。

### `AutoGenEditDialog.vue`（新）
- props/emits 同上。draft 完整副本。
- 模板：开关 + 策略 + 时机 + `currentStrategy.paramDesc != null` 时参数输入。
- 保存：`Object.assign(field, draft)` -> 关闭。无提示。

### `FieldsTab.vue`（改）
- 删 FieldDetailDialog import/状态/"详情"列。
- 加 bizType/autoGen 弹窗状态 + `onCellClick(row, column)`：按 `column.label`（"业务类型"/"自动生成策略"）路由，readOnly return。
- 业务类型/自动生成单元格：`:class-name="store.readOnly ? '' : 'cursor-pointer'"`（整格光标），span 蓝色 hover:underline。
- 自动生成单元格：timing 图标（INSERT `i-mdi-plus` 蓝 / INSERT_UPDATE `i-mdi-sync` 绿，文字前）。
- 列固定：selection/drag/#/编码 `fixed="left"`。
- `addField`：空 code/name/prop + VARCHAR(32)，nextTick focus 新行 code（`.el-table__fixed .code-cell input`）。
- `onDataTypeChange(field, newDt)`（async，`:model-value` 受控）：兼容 -> `applyDataType`；不兼容 -> confirm（确认清 bizType + applyDataType / 取消不动）。

## 共享逻辑（R9）

`app/src/utils/bizType.ts`：
- `bizTypeSupports(def, dt): boolean`
- `applyDefaults(target, def, dt): void` -- 填 defaultLength/defaultPrecision/defaultScale

## 行内切 dataType 流程（D4）

`:model-value="row.dataType"` 受控 + `@change="(newDt) => onDataTypeChange(row, newDt)"`，确认前不写 field.dataType。

```
onDataTypeChange(field, newDt):
  判断兼容性(field.dataType 仍是旧值):
    Enum && newDt != VARCHAR -> 不兼容
    普通 bizType && !bizTypeSupports(def, newDt) -> 不兼容
  if 不兼容:
    confirm("切换将清除业务类型. 是否切换?")
      确认 -> 清 bizType/bizTypeData/enum + applyDataType(field, newDt)
      取消 -> 不动(dataType 保持旧值)
    return
  applyDataType(field, newDt)  // 兼容或无 bizType

applyDataType(field, newDt):
  field.dataType = newDt
  cleanupDataType(field, newDt)         // §3.1 清理
  if bizType && !Enum: applyDefaults(field, def, newDt)
  // 全局默认兜底(bizType 未定义时):
  if VARCHAR && length==null: length=32
  if DECIMAL: precision==null->10, scale==null->4
```

## 抽取来源（从 FieldDetailDialog 迁移）

- BizTypeEditDialog：bizType/enum/bizTypeData computed + 函数（getBizTypeDataValue/setBizTypeDataValue/cleanBizTypeData/initBizTypeData/onBizTypeChange + 新 onDataTypeChange/showDataTypeSelect）+ COLORS + 模板业务类型段。
- AutoGenEditDialog：autoGenStrategies/currentStrategy + 模板自动生成段。
- 删除原 FieldDetailDialog 的 dead code（enumMode/syncEnumMode/onEnumModeChange），新组件不带。

## 写回策略（D2/D3）

- draft = `JSON.parse(JSON.stringify(field))` 完整副本。
- 保存 `Object.assign(field, draft)` 全量写回。弹窗 modal 期间行内不可编辑，未展示字段值不变，全量写回不丢数据。
- 两弹窗不编辑 code，无 code 级联（行内 `onCodeChange` 负责）。

## 校验保留

- enum 仅 VARCHAR（onBizTypeChange 已强制，兜底校验）。
- hasCode=true 时枚举值 code 必填。

## 兼容/回归

- FieldDetailDialog 仅 FieldsTab 引用（已确认），删除无外部影响。
- 既有行内编辑、拖拽、拷贝粘贴、删除级联均不动。
- 行内 onDataTypeChange 行为变化：从"直接清 bizType + warning"改为"confirm 让用户决定 + 受控取消不变"。
