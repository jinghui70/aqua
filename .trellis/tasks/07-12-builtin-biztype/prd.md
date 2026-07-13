# 内置业务类型配置文件加载

## Goal

外置 JSON 清单加载内置业务类型,与项目自定义 bizType 合并展示;内置条目只读、不可删改。本任务只做加载机制,清单初始为空,后续单独维护。

## Requirements

- **清单文件**:`src-tauri/resources/builtin-biztypes.json`,结构 `{"bizTypes": [...]}`。初始含一个示例条目 `Date`(format 参数,default "YYYYMMDD",VARCHAR/defaultLength 8),作为格式参照并验证机制。
- **参数默认值机制**:扩展 `BizTypeDataField` 加可选 `default: string | number` 字段;选 bizType 时用各 field 的 default 初始化 bizTypeData(单 field 存值、多 field 存对象)。顺带修复切 bizType 不清旧 bizTypeData 的既存缺陷。
- **资源配置**:`tauri.conf.json` 的 `bundle.resources` 注册该文件,dev 与 prod 均可读。
- **加载 command**:`builtin_biztypes_load` 无状态 command,读资源文件反序列化为 `Vec<BizTypeDefine>` 返回前端。文件缺失/JSON 非法时返回明确错误(不静默空)。
- **前端 store**:新增内置 bizType 状态,App 启动加载一次;与项目自定义 bizType 分离存储(来源即区分,不改 `BizTypeDefine` 顶层结构、不加 builtin 字段)。
- **BizTypeManage 合并展示**:内置条目(只读,无删/编辑按钮)+ 自定义条目(可增删改)。新建自定义 bizType 时校验重名需含内置 code。
- **FieldDetailDialog 下拉**:bizType 选项合并内置 + 自定义;类型↔bizType 联动与默认值填充对内置同样生效。
- **向后兼容**:`default` 可选字段,旧 schema.json 无该字段照常反序列化。

## Acceptance Criteria

- [ ] 启动不报错;`builtin-biztypes.json` 存在且为合法 JSON,含 Date 示例。
- [ ] BizTypeManage 列表显示 Date(只读、无删按钮);FieldDetailDialog 下拉可选 Date。
- [ ] 选 Date 后,format 参数自动填 "YYYYMMDD"(default 生效),dataType 锁定 VARCHAR、length 填 8。
- [ ] 内置条目不可删改;自定义 bizType 增删改不影响内置。
- [ ] 新建自定义 bizType 与内置(含 Date)重名时被拒绝。
- [ ] 资源文件缺失或 JSON 非法时,`builtin_biztypes_load` 返回明确错误信息。
- [ ] 旧 schema.json(无 default 字段)正常加载,clippy 0 warning,pnpm build 通过。

## Notes

- 存储方式:外置资源文件(已与用户确认),非嵌入二进制。终端用户理论上可改文件,接受此权衡。
- 清单内容:本任务先空,只做机制。具体 bizType 清单后续单独维护。
- 不在 `BizTypeDefine` 加 `builtin` 字段:来源(内置 store vs 项目 store)即区分,避免污染数据模型。
- 内置 bizType 被产品升级删除导致字段引用悬空的检查,留待清单有内容时再做。
