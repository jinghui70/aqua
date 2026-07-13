# 项目信息设置(中文名+basePackage)

## Goal

Project 加中文名字段;新增"项目设置"对话框编辑中文名与 basePackage;WorkspaceHome 显示中文名。

## Requirements

- **中文名字段**:`Project` 加 `name: Option<String>`(向后兼容旧 schema,无则 None)。Rust + TS 同步。
- **项目设置对话框**:菜单"配置"下加"项目设置"项(置顶),打开对话框,含中文名 + basePackage 两个输入框,保存写回 currentProject。
- **显示**:WorkspaceHome 用中文名作为项目标识(无中文名时 fallback basePackage)。
- **新建项目**:`newProject` 默认 name=None(用户在项目设置填);或默认"新项目"。
- **分组不纳入**:分组在树上编辑,本对话框不含分组管理。

## Acceptance Criteria

- [ ] 旧 schema.json(无 name)正常打开,name 为 None,不报错。
- [ ] 项目设置对话框可编辑中文名与 basePackage,保存后生效。
- [ ] WorkspaceHome 显示中文名(无则 fallback basePackage)。
- [ ] basePackage 变更后影响 Java/StrConst 生成的 package。
- [ ] pnpm build 通过。

## Notes

- 配置集成分步:本任务只补项目信息缺口。业务类型/枚举/数据集保持现有标签页,后续是否搬进统一对话框单独评估。
- 分组编辑在树上,不进本对话框(用户确认)。
