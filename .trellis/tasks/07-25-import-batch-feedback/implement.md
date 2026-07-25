# 执行计划 — 导入批量化(A)+ 按钮 loading(C)

依赖顺序:Java 契约先行(定响应结构)→ Rust 消费 → 前端止血(独立,可并行/随时)。

## Step 1 — connector 加 `importTables` action
- 文件:`connector/src/main/java/com/aqua/connector/Main.java`
- `dispatch` switch 加 `case "importTables"`:遍历 `root.get("tables")`,对每个表名调 `dialect.getColumns(conn, name)` + `dialect.getIndexes(conn, name)`,组装 `{tables:[{name,columns,indexes}]}`(columns/indexes 用 `addPOJO`,与 getColumns/listIndexes 一致)。
- 任一表反解抛异常 → 冒泡到 `main` 的 catch → `writeError`(all-or-nothing)。
- 验证:`H2DialectTest` 旁加多表 case,或临时 `echo '{"action":"importTables","tables":[...],...}' | java -jar` 手测。

## Step 2 — Rust `TableMeta` 类型
- `driver/types.rs`:加 `TableMeta { name, columns, indexes }`。
- `driver/mod.rs`:`pub use types::{..., TableMeta}`。

## Step 3 — Trait 默认方法
- `driver/trait_def.rs`:加 `async fn import_tables(&self, tables:&[String]) -> Result<Vec<TableMeta>>` 默认实现(逐表 get_columns+list_indexes)。import `TableMeta`。
- 编译即验证 native 自动获得该方法。

## Step 4 — JDBC 覆写
- `driver/jdbc.rs`:`impl Driver for JdbcDriver` 加 `import_tables` 覆写,一次 `self.call("importTables", Some(json!({"tables":tables})))`,复用 `parse_column_meta`/`parse_index_meta` 逐表解析响应 `tables` 数组。

## Step 5 — 编排改批量
- `import/from_db.rs`:`import_from_db` 改调 `driver.import_tables(&names)`,按 name→meta 映射组 `Table`(复用 `column_to_field`/`index_meta_to_index` + `TableInfo.comment` 回退)。删逐表串行 for 与 `import_table`(或改为纯组装 helper,不再触驱动)。
- 保留并跑通现有单测。

## Step 6 — 前端 loading(C,独立)
- `ImportWizard.vue`:`importing` ref + try/finally 包 `doImport`;导入按钮 `:loading`/`:disabled`;dialog `:close-on-press-escape="!importing"`;上一步按钮 `:disabled="importing"`。

## Step 7 — 验证收口
- `cd crates/aqua-core && cargo test`。
- `cd connector && mvn test`(或项目既定构建命令)。
- 前端 `pnpm -C app build`(tsc 通过)。
- 手工:JDBC 源导 ≥3 表,核对列/索引/注释;确认单次 spawn。

## 风险
- **响应体变大**:多表列元数据一次性返回。设计工具场景表数有限(几十),JSON 体量可控,不预设分页。
- **all-or-nothing**:一表失败整批失败,与旧 `?` 短路语义一致,符合"导入要么完整要么明确报错"。
