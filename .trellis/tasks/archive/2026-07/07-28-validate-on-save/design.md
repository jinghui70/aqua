# 设计

## 架构边界

- **清理**：前端 `store.saveProject` 内，直接改 `currentProject` 内存态（UI 即时反映删除）。
- **校验**：后端 `validate_project` 扩展（复用 `ValidationError{path,message}` + 不短路），前端 `tauri.projectValidate` invoke 拿 `ValidationError[]`。
- **confirm/alert**：前端 `ElMessageBox`。
- **关键字**：后端 `keywords.rs` const 集合（SQL 并集 + Java）。

## 清理规则（前端 R1）

```ts
function cleanupProject(p: Project) {
  for (const t of p.tables) {
    t.fields = t.fields.filter((f) => !!f.code?.trim());
    const validCodes = new Set(t.fields.map((f) => f.code));
    for (const idx of t.indexes ?? []) {
      idx.fields = idx.fields.filter((f) => !!f.code?.trim() && validCodes.has(f.code));
      const seen = new Set<string>();
      idx.fields = idx.fields.filter((f) => (seen.has(f.code) ? false : (seen.add(f.code), true)));
    }
    t.indexes = (t.indexes ?? []).filter((idx) => idx.fields.length > 0);
  }
}
```

## 校验规则（后端 validate_project，R2/R12）

path 格式：字段 `{table.code}.{field.code}.属性`，索引 `{table.code}.[索引名]`。

- code 表内重复（空跳过）/ code SQL 保留字 / prop Java 关键字 / prop 表内重复（空跳过）。
- prop/name 空 -> `不能为空`。
- VARCHAR length 空 / DECIMAL precision·scale 空 -> `不能为空`。
- DECIMAL precision < scale -> `precision(p) 不能小于 scale(s)`。
- 索引名空 -> `索引名不能为空`；索引名表内重复 -> `索引名重复`；索引 fields+unique 完全相同 -> `索引重复(字段与唯一性与已有索引相同)`。
- 既有：数据类型属性多余、enum 仅 VARCHAR、enum values 非空、hasCode code 必填。

## 关键字（R3，D4 并集）

`crates/aqua-core/src/schema/keywords.rs`：
- `SQL_RESERVED`（ANSI + MySQL/PG/Oracle/H2 reserved 并集，~400，大写）。
- `JAVA_KEYWORDS`（JLS ~50，小写）。
- `is_sql_reserved(code)`（大写化查，大小写不敏感）/ `is_java_keyword(prop)`（精确，大小写敏感：class 是关键字，Class 不是）。

## saveProject 接入（R4/R5/R6）

```ts
async function saveProject(path?: string): Promise<boolean> {
  if (!currentProject.value) return false;
  const target = path ?? currentPath.value;
  if (!target) throw new Error("未指定保存路径");
  cleanupProject(currentProject.value);
  const errors = await tauri.projectValidate(currentProject.value);
  if (errors?.length) {
    const ok = await showValidateErrors(errors);
    if (!ok) return false;
  }
  await tauri.projectSave(target, currentProject.value);
  // ...原 recent/dirty/gitignore/datasource/copyDatasets 逻辑不变...
  return true;
}
```

`showValidateErrors`：`ElMessageBox.confirm`，`formatErrorsHtml` 按表分组（path 首段为表 code），条目 path 去表 code 前缀 + message，按钮"仍保存"/"取消去修"。

## project_open（R10，D7）

后端 `project_open` 改 `Project::from_json`（只反序列化，结构坏才报错），不再 `parse_project`（不校验）。前端 `openProject` 末尾 `projectValidate`，有错 `ElMessageBox.alert`（"知道了"，不阻止打开）。

## IndexTab（R8/R9）

- 索引字段 el-select 选项 = `fieldCodes().filter(c => c && !row.fields.some((o, oi) => oi !== fi && o.code === c))`（预防重复选择）。
- `addIndex` 默认 name = `IDX_<表>_<序号>`，序号 = 扫描已有 `IDX_<表>_N` 最大 +1。
- 去掉 autoName 预览，name placeholder "必填"，readOnly 空名显示 `-`。

## project_validate 命令（R4）

后端 `project_validate` 返回 `Result<Vec<ValidationError>, String>`（空 Vec=合法）。前端 `useTauri.projectValidate` `invoke<ValidationError[]>`。

## 调用方适配（R6）

- `useMenuActions.doSave`：`if (!ok) return`（取消不提示已保存）。
- `NewProjectDialog`：`if (!ok) return`（取消不进入项目）。
- `confirmIfDirty`：`return await saveProject(target)`（取消返回 false，不继续关闭/切换）。

## 删 auto_index_name（R11）

`generators/ddl/index.rs` 删 `auto_index_name` 函数，`generate_index` 空名 `unwrap_or_default`（校验保证不触发）。fixture `valid-full.json` 索引补 name。

## 显示（R12，D8）

`formatErrorsHtml`：按 path 首段（表 code）分组，每表标题 `<表> (N)`，条目 path 去表 code 前缀 + message。message 简洁（不能为空 / 重复: 值 / 索引名重复 / 索引重复(...)）。HTML 转义防注入。

## 兼容/回归

- 既有 validate_project 规则（数据类型属性/enum）保留不回归（单测覆盖）。
- saveProject 落盘后逻辑不变。
- project_open 不再校验，旧项目（有校验问题）能打开 + 提示。
- 删 auto_index_name 后，空名索引 DDL 生成空名（校验保证不触发；旧项目打开后校验提示修）。
