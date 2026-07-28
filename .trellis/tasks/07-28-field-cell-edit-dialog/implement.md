# 实现计划

## 步骤

1. 抽 `app/src/composables/useBizTypeLinkage.ts`：`bizTypeSupports(def, dt)` + `applyDefaults(target, def, dt)`（从 FieldDetailDialog 迁移）。
2. 新建 `app/src/views/table-editor/BizTypeEditDialog.vue`：从 FieldDetailDialog 抽取业务类型段（computed/函数/模板/COLORS），draft 副本 + 保存写回 + cleanBizTypeData + 校验；`bizTypeSupports`/`applyDefaults` 走 composable。
3. 新建 `app/src/views/table-editor/AutoGenEditDialog.vue`：从 FieldDetailDialog 抽取自动生成段（autoGenStrategies/currentStrategy/模板），draft 副本 + 保存写回。
4. 改 `FieldsTab.vue`：
   - 删 FieldDetailDialog import（line 10）、`detailVisible/detailField/openDetail`（line 59-65）、"详情"列（line 335-339）、`<FieldDetailDialog>`（line 342）。
   - 加 `bizTypeVisible/bizTypeField/openBizType`、`autoGenVisible/autoGenField/openAutoGen` 状态与函数。
   - 业务类型单元格（line 297-301）：readOnly 只读文本；否则可点击 span 调 `openBizType(row)`。
   - 自动生成单元格（line 318-322）：同上调 `openAutoGen(row)`。
   - 改 `onDataTypeChange`（line 104-119）：清理后补 bizType 兼容联动（Enum 非 VARCHAR / 普通 bizType 不支持 -> 清空 bizType/bizTypeData/enum + ElMessage.warning；兼容 -> applyDefaults 填默认值）。引入 composable + BizTypeDefine 列表。
   - 挂两个新弹窗。
5. 删除 `FieldDetailDialog.vue`。
6. `pnpm dev` 验证。

## 验证命令

- `pnpm dev`（前端）跑通编译。
- 手测路径：
  - 新增字段 -> 点业务类型单元格 -> 选 Enum -> 配枚举值 -> 保存 -> 单元格显示 Enum，dataType 变 VARCHAR。
  - 选普通 bizType -> 确认 dataType 校正 + 默认长度精度填充。
  - 点自动生成单元格 -> 开关 -> 选策略/时机/参数 -> 保存 -> 单元格刷新。
  - 取消弹窗 -> 原字段不变。
  - 只读模式 -> 两单元格不弹窗。
  - 行内改 code -> 索引级联正常。
  - hasCode=true 枚举值 code 空 -> 保存报错。
  - 行内改 dataType 成兼容类型 -> 按 bizType 填默认长度精度。
  - 行内改 dataType 成不兼容类型 -> 清空 bizType/bizTypeData/enum + warning。
  - Enum 字段行内改非 VARCHAR -> 清 Enum + warning。

## 风险点 / 回滚

- 抽取漏函数：对照 FieldDetailDialog 逐个核对 cleanBizTypeData/applyDefaults/onBizTypeChange/isEnumBizType 等，勿漏。
- onDataTypeChange 改造：保留原清理 switch，仅追加 bizType 联动；勿破坏 VARCHAR/DECIMAL 的 length/precision/scale 清理。
- Object.assign 全量写回：确认弹窗期间行内不可编辑（modal 遮挡 + :close-on-click-modal=false），draft 未展示字段不变。
- readOnly 判断：两单元格原本无 readOnly 分支，新增时勿破坏只读文本展示。
- 回滚：git revert，FieldDetailDialog.vue 在 git 历史可恢复。
