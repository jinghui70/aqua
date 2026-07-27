# 字段拷贝与粘贴

## Goal

在表编辑器字段 Tab 提供跨表字段复用能力:多选字段 -> 拷贝到全局剪贴板 -> 切到另一表粘贴,避免重复设计相似字段(如审计字段四件套)。顺带精简操作列(拷贝/删除统一到顶部)。

## Background

- 现有 `app/src/views/table-editor/FieldsTab.vue:139 copyField` 是**原地拷贝**:深拷贝当前字段、code 加 `_COPY`、插入下一行。无剪贴板、无 paste、无跨表。
- `Field` 无 id(`app/src/types/schema.ts`),靠 code 标识,可 JSON 深拷贝。
- store 无字段级 add(`FieldsTab` 直接 `props.fields.push`);有 `renameFieldCode` / `removeFieldFromIndexes` 做索引级联。
- 操作列现有:详情 / 拷贝 / 删。顶部工具栏:`+ 新增字段`。
- 表格当前无行选中态、无多选,有拖拽排序。

## Key Decisions

- **D1 跨表粘贴**:剪贴板放全局 store,支持 A 表 copy -> B 表 paste。
- **D2 标准剪贴板语义**:copy 存剪贴板不原地插入,paste 才插入。现有 `copyField` 原地拷贝行为废弃。
- **D3 多选字段**:表格加 selection 列,copy 存 `Field[]`,paste 一次插入多个。
- **D4 顶部按钮 + 末尾插入**:顶部工具栏加"拷贝""粘贴""删除"按钮,paste 插入到字段列表末尾(粘贴后可拖拽调序)。
- **D5 code 冲突原子失败**:paste 前检查剪贴板所有字段 code 与目标表已有冲突(含剪贴板内部重复),任一冲突则不执行粘贴,提示冲突 code 列表。
- **D6 操作列精简 + 顶部统一**:操作列去掉"拷贝""删",只留"详情";拷贝/粘贴/删除统一顶部按钮(基于多选)。单行操作 = 勾选 + 顶部按钮。
- **D7 跨项目支持**:剪贴板独立 store,切换项目不清空,支持跨项目 paste。bizType 跨项目引用降级(目标项目无该 bizType 定义时 UI 显示 code 原值,数据照搬)。

## Requirements

- **R1 多选**:字段表格加 `type="selection"` 列;readOnly 时禁用选择。
- **R2 顶部"拷贝"**:拷贝选中字段(深拷贝)到全局剪贴板;无选中时禁用;提示"已拷贝 N 个字段"。
- **R3 顶部"粘贴"**:从全局剪贴板深拷贝字段,追加到当前表 `fields` 末尾;剪贴板空时禁用。
- **R4 原子冲突检查**:paste 前检查剪贴板所有字段 code 与目标表已有冲突(含剪贴板内部重复);任一冲突则不执行粘贴,提示冲突 code 列表。
- **R5 copy 不改原字段**:深拷贝存剪贴板,原字段不变。
- **R6 prop 跟随**:不单独处理 prop 冲突(跳过冲突 code 的字段时其 prop 一并跳过)。
- **R7 enum/bizType 照搬**:深拷贝含 `enum`(InlineEnum 内联);`bizType`(全局 code 引用)原值照搬。
- **R8 顶部"删除"**:删除选中字段,逐个 `splice` + `removeFieldFromIndexes` 级联清理索引引用;无选中禁用;提示"已删除 N 个字段"。
- **R9 操作列精简**:操作列只保留"详情"(去掉"拷贝""删")。
- **R10 readOnly 行为**:readOnly 时选择列可见可用(能选才能拷贝)、"拷贝"按钮可见可用;"粘贴""删除"按钮隐藏(不可见)。
- **R11 剪贴板独立**:剪贴板 store 独立于 project,切换项目不清空。
- **R12 bizType 跨项目降级**:跨项目 paste 时,目标项目无该 bizType 定义则 UI 显示 code 原值(现有 `bizTypeLabel` fallback),数据照搬不变。

## Acceptance Criteria

- A1 选中多个字段点"拷贝",切到另一表点"粘贴",选中字段出现在目标表末尾。
- A2 paste 时若任一 code 与目标表冲突(或剪贴板内部重复),不执行粘贴,提示冲突 code 列表。
- A3 copy 后原字段不变(不原地插入副本)。
- A4 无选中时"拷贝""删除"禁用;剪贴板空时"粘贴"禁用。
- A5 readOnly 时选择列、"拷贝"可见可用;"粘贴""删除"隐藏。
- A6 粘贴的字段可正常行内编辑、拖拽排序。
- A7 删除字段后,索引中对该字段 code 的引用被清理(`removeFieldFromIndexes` 级联)。
- A8 切换项目后剪贴板仍保留,能在新项目表 paste。
- A9 跨项目 paste 时 bizType 若目标项目无定义,显示 code 原值(数据照搬)。

## Out of Scope

- 字段模板库(保存常用字段供复用)。
- 快捷键 Ctrl+C / Ctrl+V(暂用按钮,快捷键后续)。
- paste 到指定行位置(暂末尾,靠拖拽调序)。

## Technical Notes

- 剪贴板 store:独立 `useClipboardStore`(不随项目切换清空,支持跨项目 paste),存 `Field[]`,深拷贝入、深拷贝出。
- 冲突检测:目标表 code 建 Set,paste 前检查剪贴板所有 code(含内部重复),任一冲突则中止,提示冲突 code。
- 删除级联:遍历选中,逐个 `splice` + `store.removeFieldFromIndexes(tableId, code)`。
- 顶部按钮可用性:`computed`(选中数 > 0 / 剪贴板非空 / !readOnly)。
- `copyField` 函数删除(原地拷贝废弃),操作列拷贝/删按钮删除。
