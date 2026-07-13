# 删除级联保护:枚举 + 业务类型提示改进

## Goal

删除全局枚举时统计关联字段并级联清除引用(消除悬空引用);同时把 bizType 删除提示由字段级改为表级聚合,两者体验统一。

## Requirements

- **enum 级联删除**(新):删除全局枚举 code 时,统计引用该 code 的字段(`field.enum === code`,string 引用),按表聚合提醒,确认后级联清除:
  - `field.enum = undefined`
  - 若该字段 `bizType === "Enum"`,一并清 `field.bizType`(避免 Enum bizType 无 enum 的不一致)
  - 内联枚举(`field.enum` 为对象)不受全局枚举删除影响
- **bizType 删除提示改进**:现有字段级提示(表code.字段code 列表)改为表级聚合。
- **统一提醒格式**(bizType 与 enum 共用):
  ```
  {类型} {code} 被以下 {N} 张表使用:
  {表名列表(前 8 个,超出省略)}
  (共 {M} 个字段引用,删除将清除这些字段的{业务类型|枚举}设置)
  确认删除?
  ```
  未被引用时:简单 `确认删除{类型} {code}?`。
- **级联清除范围**:
  - bizType:清 `field.bizType` + `field.bizTypeData`(已有,保持)
  - enum:清 `field.enum` + 若 `bizType==="Enum"` 则清 `field.bizType`

## Acceptance Criteria

- [ ] 删除被引用的全局枚举:弹窗列出引用它的表(表级),确认后这些字段的 enum(及 Enum bizType)被清除,无悬空引用。
- [ ] 删除被引用的 bizType:弹窗改为表级聚合(不再是字段列表)。
- [ ] 内联枚举字段不受全局枚举删除影响。
- [ ] 未被引用的枚举/bizType 删除时简单确认,不级联。
- [ ] 提示格式两者一致;表数 >8 时省略。
- [ ] pnpm build 通过。

## Notes

- bizType 级联清除逻辑已实现(上一任务),本任务只改提示聚合方式 + 新增 enum 级联。
- enum 级联清 bizType 的理由:field.bizType="Enum" 必须配 field.enum;enum 被清后 bizType 留空更一致,用户可重新指定。
- 纯前端改动,无后端/command 变更。
