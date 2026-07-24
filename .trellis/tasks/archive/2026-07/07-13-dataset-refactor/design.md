# Design - 数据集重构

## 1. 文件规则 + 目录扫描

- 文件:`{项目目录}/{主文件名}.{数据集名}.data`(JSONL 格式,后缀 .data)
  - 主文件名 = project.aqua -> "project";数据集名 = "dev" -> `project.dev.data`
- 内容:JSONL 每行 `{"table":"SYS_USER","row":{"ID":1,"NAME":"admin"}}`,不存表结构(结构用主项目)
- Rust command `scan_datasets(project_path) -> Vec<DatasetInfo>`:
  - 取项目文件名(去 .aqua)作前缀
  - 扫描同目录匹配 `{前缀}.*.data`
  - 返回 `DatasetInfo { name }`(无 format,统一 .data)
- 前端:进数据集页 invoke scan_datasets -> 下拉

## 2. 新建数据集

- 弹窗:录入数据集名(统一 .data,无类型选择)
- Rust command `create_dataset(project_path, name)`:
  - 文件 = `{dir}/{主文件名}.{name}.data`
  - 写空文件(0 字节,无行)
- 新建后刷新下拉 + 选中

## 3. 编辑(DBeaver 模式)

- 选数据集 -> `dataset_load(path, project)` -> 行数据到表格(可编辑)
- **dirty 跟踪**:rowsMap 修改时 `dirty = true`;watch rowsMap deep
- **dirty 时显示保存/取消按钮;无修改隐藏**
- 保存:`dataset_save(path, project, entries)` + `dirty = false`
- 取消:恢复原始 rowsMap(深拷贝)+ `dirty = false`
- 无打开按钮(选下拉即加载);无保存按钮(dirty 时才显示)

## 4. 导入(数据库 -> 数据集)

- 选数据源(下拉,已配)+ 选表(TableSelectDialog 复用)
- `Driver::query_table_rows(table) -> Vec<Vec<Value>>`(新增 trait 方法)
- 数据集覆盖:每表 clear + insert_rows
- `dataset_save`(写文件)
- 前端:导入按钮 -> 选数据源 + 选表 -> invoke import_from_db

## 5. 导出(数据集 -> 数据库)

- 选数据源 + 选表(TableSelectDialog)+ **覆盖提醒**(ElMessageBox)
- 数据集 `read_table_rows(table) -> Vec<Map>`
- `Driver::execute_update("TRUNCATE TABLE {table}")`
- `Driver::execute_update("INSERT INTO {table} (col1, col2, ...) VALUES (v1, v2, ...)")`(批量)
- 前端:导出按钮 -> 选数据源 + 选表 + 提醒 -> invoke export_to_db

## 6. DDL 导出 + 数据集 INSERT

- DdlExportDialog 加**数据集下拉**(scan_datasets)
- 选中数据集时,DDL 输出追加该数据集的 INSERT 语句(按选中表)
- 未选 -> 只 DDL
- Rust:`generate_ddl` 加 `dataset: Option<&Dataset>` 参数;有 dataset 时追加 INSERT
  - 或独立 `generate_insert(dataset, tables) -> String`,DDL 后拼接

## 7. 底层写(Driver trait 扩展)

- 加 `query_table_rows(table) -> Result<Vec<Vec<serde_json::Value>>>`(查,导入用)
- 加 `execute_update(sql) -> Result<usize>`(执行 TRUNCATE/INSERT,导出用)
- native(MySQL/PG):
  - mysql_async: `conn.exec(sql, params)` / `conn.query_drop(sql)`
  - tokio-postgres: `client.execute(sql, &[])` / `client.query(sql, &[])`
- JDBC(connector.jar):
  - Main.java 加 action `queryRows(table)` -> `{rows: [[v1,v2,...], ...]}`
  - Main.java 加 action `executeUpdate(sql)` -> `{affected: N}`
  - AbstractJdbcDialect 加 `queryRows(conn, table)` / `executeUpdate(conn, sql)`

## 8. connector.jar 扩展

- Dialect.java 加 `queryRows(Connection, table)` / `executeUpdate(Connection, sql)`
- AbstractJdbcDialect 实现:
  - `queryRows`: `SELECT * FROM {table}` -> 遍历 ResultSet -> JSON 数组
  - `executeUpdate`: `stmt.executeUpdate(sql)`
- Main.java dispatch 加 case `queryRows` / `executeUpdate`
- Rust JdbcDriver 实现 `query_table_rows` / `execute_update`(spawn connector action)

## 影响面

- **Rust**:Driver trait(加 2 方法)+ mysql.rs/postgres.rs(实现)+ jdbc.rs(spawn queryRows/executeUpdate)+ dataset mod(**重写**:JSONL 读写,去 SQLite/rusqlite)+ DDL 生成器(INSERT)+ commands(scan_datasets/create_dataset/import_from_db/export_to_db)
- **Java**:Main.java + AbstractJdbcDialect(queryRows/executeUpdate)
- **前端**:DatasetManage 重构(下拉+dirty+导入导出)+ DdlExportDialog(数据集下拉)+ 新建弹窗 + TableSelectDialog 复用

## 风险

- JDBC 写(executeUpdate)需 connector.jar 扩展,重新打包
- native 写(mysql_async/tokio-postgres execute)需确认 API
- TRUNCATE 需 DB 权限
- 数据集 dirty 跟踪(深层 watch rowsMap)
- 大数据集 INSERT 批量(事务?分批?)

## 验证

- cargo test/clippy(Driver trait + dataset + DDL)
- mvn test(connector queryRows/executeUpdate)
- vue-tsc + vite build
- 手动:新建数据集 + 编辑保存 + 导入导出 + DDL+INSERT
