# 业务类型编辑优化(定义端 + 使用端)

## Goal

优化业务类型的两端交互与展示:**定义端**(BizTypePanel,配置中心内)的新建/编辑/只读展示,和**使用端**(FieldDetailDialog 字段编辑)的 bizType 选择/bizTypeData 录入/枚举/布局。让业务类型定义清晰、只读美观、使用端录入符合直觉。

## 背景

业务类型(BizType)两端:
- **定义端** BizTypePanel:左列表(内置+自定义)+ 右编辑(code/name/description/supportedDataTypes/bizTypeData.fields)。当前 code 用 disabled input、只读用 disabled 灰框、参数表头英文、数据类型列宽不足、列表删按钮只读时置灰。
- **使用端** FieldDetailDialog:选 bizType(下拉 option 显示英文 code)+ bizTypeData 动态表单 + 枚举(无/内联 radio)。当前下拉 label 英文、枚举来源 radio 冗余、divider 分块、约束位置、switch 文字。

两个只读来源:全局只读(store.readOnly,打开项目默认)+ 预置只读(builtin.isBuiltin)。任一只读 -> 编辑框 readonly(非 disabled,不灰)。

## 需求

### 定义端(BizTypePanel)

1. **新建**:弹窗录 code + name(当前 ElMessageBox.prompt 只录 code,name 默认=code)。改为录两字段。
2. **编辑 code**:只读文本/tag(不用 input)。当前 disabled input。
3. **编辑 name/描述**:name 可编辑(已可),描述改 textarea 加长(当前单行 input)。
4. **两个只读(全局 + 预置)**:编辑框用 `readonly`(非 `disabled`,不灰);数据类型列表 + 参数列表只读时显示正常只读表格(非编辑组件)。
5. **无参数**:bizTypeData.fields 为空则隐藏参数表格(当前空表格仍显示)。
6. **参数列表拖拽**:bizTypeData.fields 可拖拽排序(参考 FieldsTab Sortable)。
7. **参数列表表头中文**:当前 name/type/description/required/default 英文,改中文。
8. **数据类型列表宽度**:字段宽度不够,加宽。
9. **全局只读时列表"删"按钮不可见**:当前 `:disabled`,改 `v-if="!store.readOnly"`(预置本就无删按钮)。
10. **supportedDataTypes 表格可拖拽排序**:参考参数列表/字段拖拽(Sortable)。

### 使用端(FieldDetailDialog)

10. **bizTypeData 录入两列**:label 用参数 `description`,默认值作 `placeholder`(当前 label 用 field.name)。
11. **空/默认值不保存**:用户没输入或输入等于默认值 -> 不存 bizTypeData(输出 JSON 也不输出,skip)。
12. **单属性 -> 值,多属性 -> 对象**:业务类型只有 1 个 bizTypeData field 时,bizTypeData 输出为该属性值(非对象);多 field 输出对象。
13. **bizType 下拉 option 显示中文**:当前 option label = bizType(英文 code),改 name(中文)。
14. **删"枚举来源" radio**:选 Enum 业务类型即内联枚举(自动 inline);无枚举 = 不选 Enum。删 enumMode radio(无/内联)。
15. **对话框不用 divider 分块**:当前 el-divider 分块,去掉。
16. **约束(主键/非空)移到类型后**:当前约束在别处,移到类型字段后。
17. **自动生成 switch 文字**:"启用" -> "自动生成"。
18. **bizType 下拉 form-item label 改"业务类型"**:当前 label "bizType"(英文),改"业务类型"。

### Java 生成(entity.rs)

19. **bizType=Bool -> Java boolean**:字段 bizType 为 Bool 时,Java 类型用基本类型 `boolean`(而非包装 Boolean)。
20. **Clob/Blob -> @Column(sqlType=Types.BLOB)**:字段 dataType 为 Clob 或 Blob 时,Java 字段加 `@Column(sqlType=Types.BLOB)` 注解。

## 验收标准

- [ ] BizTypePanel 新建弹窗录 code + name
- [ ] 编辑 code 只读文本/tag(非 input),name 可编辑,描述 textarea 加长
- [ ] 全局只读 / 预置只读 时,编辑框 readonly(非 disabled),数据类型/参数列表只读表格展示
- [ ] 无参数隐藏参数表格
- [ ] 参数列表可拖拽排序
- [ ] 参数列表表头中文
- [ ] 数据类型列表字段宽度足够
- [ ] 全局只读时列表"删"按钮 v-if 不可见
- [ ] FieldDetailDialog bizTypeData 录入两列(label=description,placeholder=default)
- [ ] 空/默认值不保存(输出 JSON 不输出)
- [ ] 单属性 bizTypeData 输出值,多属性输出对象
- [ ] bizType 下拉 option 显示中文 name
- [ ] 删"枚举来源" radio(选 Enum 自动内联)
- [ ] 对话框无 divider 分块
- [ ] 约束(主键/非空)在类型后
- [ ] 自动生成 switch 文字"自动生成"
- [ ] bizType 下拉 form-item label "业务类型"
- [ ] supportedDataTypes 表格可拖拽排序
- [ ] bizType=Bool 字段 Java 用 boolean
- [ ] dataType=Clob/Blob 字段 Java 加 @Column(sqlType=Types.BLOB)
- [ ] cargo test/clippy + vue-tsc 全过,现有功能不回归

## 非目标

- 业务类型 schema 结构改造(BizTypeDefine 不变)
- 内置 bizTypes 内容调整
- 级联逻辑改造

## 技术笔记

- 只读展示分两类:**name/描述**(表单字段)用 `readonly` input(不灰,可选中);**数据类型列表/参数列表**(表格)用 span 纯文本(正常只读表格,无编辑框),参考 FieldsTab 的 v-if span / v-else 编辑组件模式。
- 单属性/多属性 bizTypeData:field.biz_type_data 是 `Option<serde_json::Value>`,前端控制单值/对象(当前已部分支持,需确认空/默认跳过)。
- 空值/默认值不保存:前端编辑时,biz_type_data 全空或全等于默认 -> undefined(序列化 skip_serializing_if None 已有)。
- 参数拖拽:复用 FieldsTab 的 Sortable 模式(handle + onEnd splice)。
