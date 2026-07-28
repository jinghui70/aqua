# 字段编辑改为列表单元格弹窗

## Goal

去掉笨重的"字段详情对话框"，把表格里两个只读单元格（业务类型、自动生成）改成可点击入口，点击弹出聚焦的小对话框编辑。其余字段维持行内编辑不变。

## Background

- `FieldsTab.vue`：字段表格，行内编辑 code/prop/name/类型/主键/非空/默认值/备注；业务类型、自动生成两列只读 `<span>`；末列"详情"按钮开 `FieldDetailDialog`。
- `FieldDetailDialog.vue`：全景编辑弹窗（基本属性 + 自动生成 + 业务类型 + 备注），draft 副本 + 保存写回。仅 FieldsTab 引用，已删除。
- 行内改类型原只清理不适用属性；完整"类型↔bizType 双向联动"（design.md §3.4）只在 FieldDetailDialog 内发生，现已在行内 + 弹窗双向补齐。
- design.md §3.4：选 bizType 校正/过滤 dataType 并填默认 length/precision/scale；Enum 强制 VARCHAR；双向过滤。

## Requirements

- R1 删除 `FieldDetailDialog.vue`；FieldsTab 移除"详情"列、按钮、detail 状态与 import。
- R2 "业务类型"单元格：`@cell-click` 整格可点（cursor-pointer），弹 `BizTypeEditDialog`；只读模式不弹。
- R3 "自动生成策略"单元格：同 R2，弹 `AutoGenEditDialog`。
- R4 `BizTypeEditDialog`：bizType 下拉 + Enum 内联枚举 + bizTypeData 动态表单（default 初始化、空/默认值清理）+ dataType 下拉（该 bizType 支持多类型时显示，选中填默认值）。
- R5 `AutoGenEditDialog`：开关 + 策略 + 时机 + 策略参数（paramDesc 非空时显示）。
- R6 bizType↔dataType 双向联动：
  - 弹窗内 `onBizTypeChange` 校正 dataType（Enum->VARCHAR、不支持->换首个支持）+ 填默认值；`onDataTypeChange`（弹窗内）清理 + applyDefaults。
  - 行内切 dataType：兼容 -> applyDefaults + 全局默认兜底；不兼容（含 Enum 改非 VARCHAR）-> `ElMessageBox.confirm`，确认清 bizType、取消则 dataType 不变（`:model-value` 受控，确认前不写）。
- R7 草稿+保存：draft 副本，保存 `Object.assign` 写回，取消可回退；保存时不弹"已保存"提示（校验失败的 error 提示保留）。
- R8 code 改名级联索引（行内 `onCodeChange`）不回归；两弹窗不碰 code。
- R9 `bizTypeSupports`/`applyDefaults` 抽 `app/src/utils/bizType.ts`，弹窗与行内共用。
- R10 列固定：selection / drag / # / 编码 四列 `fixed="left"`（# 连带固定，保证连续）。
- R11 自动生成 timing 图标（文字前）：INSERT=`i-mdi-plus` 蓝、INSERT_UPDATE=`i-mdi-sync` 绿；无自动生成不显示。
- R12 列头"自动生成" -> "自动生成策略"（`onCellClick` 判断同步）。
- R13 `addField`：code/name/prop 默认空、VARCHAR length=32；新增后 nextTick focus 到新行 code 输入框（fixed 层）。
- R14 切换类型全局默认兜底：VARCHAR length=32、DECIMAL precision=10/scale=4（bizType 未定义默认值时）。

## Key Decisions

- D1（反转）业务类型弹窗含 dataType 下拉：选了非 Enum bizType 且支持 >1 种 dataType 时显示，选中填默认值。原 D1"不含 dataType"已反转。
- D2 草稿+保存模式，复用 draft/Object.assign 写回逻辑。
- D3 draft 完整副本 + `Object.assign` 全量写回（弹窗 modal 期间行内不可改，安全）。
- D4 行内切 dataType 不兼容 -> confirm 让用户决定（确认清 bizType / 取消不变）。用 `:model-value` 受控，确认前不写 field.dataType，取消时 dataType 完全不变。
- D5 UI 迭代决策：列固定 left、timing 双色图标、列头改名、addField 空默认+focus、切换默认值兜底、cell-click 整格可点。

## Acceptance Criteria

- [ ] 表编辑页无"详情"按钮，`FieldDetailDialog.vue` 已删除。
- [ ] 点击业务类型/自动生成策略单元格（整格可点）弹对应对话框，保存后单元格刷新；只读模式不弹。
- [ ] 业务类型弹窗：选 Enum 强制 VARCHAR + 内联枚举；选普通 bizType 校正 dataType + 填默认值；多类型 bizType 显示 dataType 下拉。
- [ ] 行内切 dataType：兼容填默认值（VARCHAR 32 / DECIMAL 10,4）；不兼容弹 confirm，确认清 bizType、取消 dataType 不变。
- [ ] 自动生成 timing 图标：INSERT 蓝 plus、INSERT_UPDATE 绿 sync。
- [ ] selection/drag/#/编码 四列固定左侧。
- [ ] 新增字段：code/name/prop 空、VARCHAR(32)，焦点落 code 输入框。
- [ ] code 改名级联索引不回归；取消弹窗不污染原字段。

## Out of Scope

- 行内其他列编辑行为不变。
- 自动生成策略全局管理（AutoGenStrategyPanel.vue）不动。
- 业务类型管理全局配置页不动。
- 保存时合法性校验（code 重复/关键字、索引结构等）-> 新任务 `07-28-schema-validate-on-save`。
