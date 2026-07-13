# 数据集数据行读写 — 技术设计

## 边界与契约

数据集是**独立文件**,与 project schema 分离。后端**无状态**:不持有打开的数据集,每个 command 完成一次文件级读/写。行数据由前端持有(内存),编辑在前端做,保存整体写回 —— 与 `project_open`/`project_save` 一致。

### 数据结构(前后端共用 JSON 形状)

```
DatasetEntry { table: String, data: Vec<Map<String, Value>> }
数据集 = Vec<DatasetEntry>
```

行 `Value` 按 §4.5:DECIMAL/DATE/DATETIME/VARCHAR/CLOB → JSON string,INT/LONG/TINYINT → JSON number,BLOB → base64 string,空 → null。key 用字段 code(大写)。

### 后端 API(aqua-core `dataset` 模块新增)

- `load_dataset(path, project) -> Result<Vec<DatasetEntry>>` — 按扩展名分派:
  - `.json`:读文件、`serde_json` 反序列化、按 project 表结构校验(表名/字段齐全)。
  - `.db`:`Dataset::load` → 遍历 project.tables,`SELECT <cols> FROM <t>`,按字段类型把 rusqlite `Value` 转 JSON `Value`。
- `save_dataset(path, project, entries) -> Result<()>`:
  - `.json`:序列化写文件(pretty)。
  - `.db`:`Dataset::new(project)`(建表)→ 每 entry 每行 `INSERT`(参数化,按类型绑定)→ `save(path)`。
- 校验 `validate_against(project, entries)`:表必须属于 project,行 key 必须是该表字段 code。不一致返回明确错误。

SQLite 读写复用现有 `Dataset`(new/load/save/connection)。新增行转换 helper:`row_to_json(field, sqlite_value)` 与 `bind_value(field, json_value)`。

### Tauri commands(src-tauri `commands/dataset.rs` 新增)

- `dataset_load(path: String, project: Project) -> Result<Vec<DatasetEntry>, String>`
- `dataset_save(path: String, project: Project, entries: Vec<DatasetEntry>) -> Result<(), String>`

注册进 `invoke_handler`。前端 `useTauri` 加 `datasetLoad`/`datasetSave`。

### 前端 DatasetManage 改造

- 状态:`entries` (Map<tableCode, rows[]>) 内存态;`currentPath` 当前数据集文件。
- 顶部:新建(pickSaveFile 选 .json/.db)、打开(pickOpenFile)、保存(datasetSave)。
- 表树节点行数 = `entries[tableCode]?.length ?? 0`。
- 数据网格:el-table 可编辑单元格(el-input),按字段类型限制;新增行(按 schema 生成空行)、删行、清空表。
- 打开时 datasetLoad → 填充 entries;保存时组装 `DatasetEntry[]` → datasetSave。

## 取舍

- **无状态 vs 有状态连接**:选无状态。数据集数据量在设计定位是"可编辑量级",一次性载入内存可接受,避免 command 持有 SQLite 连接的生命周期管理。
- **行编辑在前端**:复用 el-table,即时响应;保存原子(整体写),避免半写状态。
- **多数据集下拉**:本期不做目录扫描,`datasets` 列表退化为"当前打开的文件名";留后续。

## 兼容性

- `Dataset` 现有 API 不改签名,仅新增模块级函数与 helper。
- 前端 DatasetManage 当前是占位 UI,全量替换,无迁移问题。
