# 数据集数据行读写 — 执行计划

## 步骤

1. **后端 dataset 行转换 helper**(aqua-core `dataset/mod.rs`)
   - `row_to_json(field, rusqlite::types::ValueRef) -> serde_json::Value`(按 DataType 转)
   - `bind_json_value(field, &serde_json::Value) -> rusqlite param`(按 DataType 绑定;DECIMAL 存 TEXT,BLOB base64 解码)
   - 验证:单元测试往返。

2. **后端文件级读写函数**(aqua-core `dataset/mod.rs`)
   - `load_dataset(path, project) -> Result<Vec<DatasetEntry>>`(扩展名分派 + 校验)
   - `save_dataset(path, project, entries) -> Result<()>`
   - `validate_against(project, entries)`(表/字段一致性)
   - `Dataset` 加 `read_table_rows(table) -> Vec<Map>` 与 `insert_rows(table, rows)` 辅助。

3. **Rust 测试**(`crates/aqua-core/tests/dataset_rows.rs` 或 mod 内)
   - JSON 往返、SQLite 往返、DECIMAL 精度、null 保留、结构不一致报错。
   - `cargo test -p aqua-core`。

4. **Tauri commands**(`src-tauri/src/commands/dataset.rs`)
   - `dataset_load` / `dataset_save`;`commands/mod.rs` 导出;`lib.rs` 注册 handler。
   - `cargo check -p aqua`。

5. **前端 useTauri**(`app/src/composables/useTauri.ts`)
   - `datasetLoad(path, project)` / `datasetSave(path, project, entries)`;类型 `DatasetEntry`。

6. **前端 DatasetManage 重写**(`app/src/views/DatasetManage.vue`)
   - 内存 entries + currentPath;新建/打开/保存(文件对话框);表树真实行数;数据网格增删改行 + 清空。
   - `pnpm run build`。

7. **实跑验证**:`pnpm dev` 建数据集→加数据→保存→重开;JSON 与 SQLite 各一遍。

## 校验命令

- `cargo test -p aqua-core`
- `cargo check -p aqua`
- `cd app && pnpm run build`

## 回滚点

- 步骤 1-3 纯 aqua-core 新增,不影响现有;可独立回滚。
- 步骤 6 全量替换 DatasetManage;保留 git 前一版本可回退。
