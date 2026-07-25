# 技术设计 — 导入批量化(A)+ 按钮 loading(C)

前置阅读:`.trellis/spec/connector/backend/subprocess-io-contract.md`(新增 action 必守 UTF-8/路径/错误协议)、`.trellis/spec/aqua-core/backend/database-guidelines.md`。

## 1. 数据流对比

**现状(JDBC 导 N 表)**
```
from_db.rs  for table in tables:
              get_columns(t)   → spawn java (JVM冷启+连库+查)   ┐
              list_indexes(t)  → spawn java (JVM冷启+连库+查)   ┘ ×N,串行 ⇒ 2N 次 spawn
```
**目标**
```
from_db.rs  import_tables(names) → spawn java 一次
              Java: 开一条连接 → for name: getColumns+getIndexes → 汇总返回
            ⇒ JDBC 恒为 1 次 spawn;native 默认循环(池化,无 JVM)
```

## 2. 跨进程契约(Rust ↔ Java)

新增 action `importTables`。请求在现有公共字段(action/dialect/host/port/user/password/database/schema/driversDir)基础上加:

```json
{ "action": "importTables", "tables": ["T_USER", "T_ORDER"], ...连接字段 }
```

响应(UTF-8,错误仍走 stdout `{error}`,遵守 subprocess-io-contract §1/§3):
```json
{ "tables": [
    { "name": "T_USER",
      "columns": [ {name,dataType,length,precision,scale,nullable,isKey,defaultValue,comment}, ... ],
      "indexes": [ {name,fields:[...],unique}, ... ] }
] }
```
- `columns`/`indexes` 元素结构**复用**现有 `getColumns`/`listIndexes` 的 POJO 序列化,零新增解析分支。
- 表顺序按请求 `tables` 顺序返回;某表反解失败 → 整体 `writeError`(保持 all-or-nothing,与旧逐表 `?` 短路一致,不静默丢表)。

## 3. Rust 侧

### 3.1 新类型(`driver/types.rs`,`mod.rs` 导出)
```rust
/// 批量反解:单表的列+索引元数据。
pub struct TableMeta {
    pub name: String,
    pub columns: Vec<ColumnMeta>,
    pub indexes: Vec<IndexMeta>,
}
```
注释不进 `TableMeta`——沿用 `from_db` 已持有的 `TableInfo.comment`。

### 3.2 Trait 加批量方法带默认实现(`driver/trait_def.rs`)
```rust
/// 批量反解多表(列+索引)。默认逐表调用(native 池化,无额外开销);
/// JDBC 覆写为单次 spawn,消除 2N 次 JVM 冷启动。
async fn import_tables(&self, tables: &[String]) -> Result<Vec<TableMeta>> {
    let mut out = Vec::with_capacity(tables.len());
    for t in tables {
        out.push(TableMeta {
            name: t.clone(),
            columns: self.get_columns(t).await?,
            indexes: self.list_indexes(t).await?,
        });
    }
    Ok(out)
}
```
→ native(MySQL/PG)**不改代码**,自动走默认实现(池化复用连接)。

### 3.3 JDBC 覆写(`driver/jdbc.rs`)
```rust
async fn import_tables(&self, tables: &[String]) -> Result<Vec<TableMeta>> {
    let resp = self.call("importTables", Some(json!({ "tables": tables }))).await?;
    // 复用 parse_column_meta / parse_index_meta 解析每表
}
```
一次 `call` = 一次 spawn。`call()` 本身不动(协议/编码/错误处理已在其中)。

### 3.4 导入编排(`import/from_db.rs`)
`import_from_db` 改为:
```rust
let names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
let metas = driver.import_tables(&names).await?;
// 按 name 建 map,for table in tables { 取 meta + table.comment → 组 Table }
```
删除逐表 `import_table` 的串行 for(`column_to_field`/`index_meta_to_index`/注释回退逻辑原样复用)。

## 4. Java 侧(`connector/.../Main.java`)
`dispatch` switch 加 `case "importTables"`:读 `root.get("tables")` 数组 → 复用 `dialect.getColumns(conn,name)` + `dialect.getIndexes(conn,name)` 循环 → 组 `{tables:[...]}`。连接由 `main()` 现有 `try(Connection conn = dialect.connect(config))` 提供,**天然一条连接**,无需改连接管理。Dialect 接口零改动。

## 5. 前端(`app/src/components/ImportWizard.vue`)
- 加 `const importing = ref(false)`;`doImport` 用 try/finally 包 `importing`。
- 「导入」按钮:`:loading="importing"`(line 203)。
- `el-dialog` 加 `:close-on-press-escape="!importing"` 且已有 `:close-on-click-modal="false"`;footer 上一步/导入按钮 `:disabled="importing"`。
- `useTauri.ts` 的 `importFromDb` 签名不变(后端 command 签名不变,仅内部批量)。

## 6. 影响面
| 层 | 文件 | 改动 |
|---|---|---|
| connector | `Main.java` | +1 case,复用现有 dialect 方法 |
| aqua-core | `driver/types.rs`,`mod.rs` | +`TableMeta` |
| aqua-core | `driver/trait_def.rs` | +默认方法 |
| aqua-core | `driver/jdbc.rs` | +覆写(1 spawn) |
| aqua-core | `import/from_db.rs` | 编排改批量 |
| app | `ImportWizard.vue` | loading 态 |

native `mysql.rs`/`postgres.rs`、Tauri command 层、`useTauri.ts` **不改**。

## 7. 验证
- aqua-core `cargo test`:`from_db` 现有单测(`column_to_field`/`index_meta_to_index`)保留;补 `import_tables` 默认实现的 mock 测试(可选)。
- connector:`importTables` 走 H2 单测(参照 `H2DialectTest`)验证多表返回。
- 手工:JDBC 数据源导 ≥3 表,比对列/索引/注释与旧逻辑一致;spawn 次数=1(可加临时 log 确认)。
