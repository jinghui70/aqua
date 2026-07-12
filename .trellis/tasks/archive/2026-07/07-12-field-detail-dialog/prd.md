# 字段编辑弹窗(enum/bizTypeData/autoGenerate 完整配置)

## 背景
查漏发现 P0 核心缺口: 字段的 enum 引用/内联、bizTypeData 值、autoGenerate.param 无编辑入口。
行内表格已很宽,这些低频复杂配置用弹窗编辑(§6.3「行内编辑 + 弹窗编辑」)。

## 目标
字段表格每行加"详情"按钮,打开字段完整编辑弹窗,补齐所有字段属性。

## 包含(补齐 §3.2 全字段)
- FieldDetailDialog: 完整字段编辑
  - 基本: code(大写)/prop/name/dataType/length/precision/scale/comment
  - 约束: isKey/notNull/defaultValue
  - autoGenerate: enabled/strategy/param/timing (补 param)
  - bizType: 下拉 + bizTypeData 值编辑(按 bizType 定义的 fields 生成表单)
  - enum(§3.5, 仅 VARCHAR): 无/引用全局/内联 三选一
    - 引用: 从 project.enums 选
    - 内联: name/hasCode/values(id/name/code?/color?) 编辑
- FieldsTab 每行"详情"按钮打开弹窗

## 范围(不含,后续)
- 数据类型↔业务类型联动过滤/默认值自动填充(P1 下一批)
- bizType 内置标记(P1)
- 拖拽/复制粘贴(P2)

## 验收标准
- [ ] FieldDetailDialog.vue: 完整字段编辑
- [ ] autoGenerate.param 输入(strategy=default 显前缀/now 显格式)
- [ ] bizTypeData 值: 按选中 bizType 的 bizTypeData.fields 动态生成表单(单field存值/多field存对象)
- [ ] enum: 无/引用/内联 三态,仅 VARCHAR 可用,内联含 values 编辑
- [ ] FieldsTab 加"详情"按钮 + 弹窗接线
- [ ] 改动写回 store(响应式)
- [ ] pnpm build 通过

## 约束
- 复用 EnumManage 的 values 编辑模式(内联枚举)
- enum 非 VARCHAR 时禁用/提示
- ElMessageBox/el-dialog, unocss px
