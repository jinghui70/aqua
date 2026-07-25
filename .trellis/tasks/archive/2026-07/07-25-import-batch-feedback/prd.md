# 数据库导入批量化与进度反馈

## Goal

修复"从数据库导入表结构"的两个体验问题:**慢**(导 N 表 = 2N 次 JVM 冷启动,串行,分钟级)与**像没反应**(导入按钮无 loading 态)。

## 根因(实测坐实,非推测)

1. **慢 = JDBC 每进程冷启动 ×2N**
   - `import/from_db.rs` 逐表调 `get_columns` + `list_indexes` 两个方法。
   - `driver/jdbc.rs` 每个方法 = 一次 `self.call()` = 一次 `java -jar connector.jar` spawn。
   - `Main.java` 每进程只开一条 JDBC 连接、干一个 action、退出。
   - 结果:JDBC 导 N 表 = **2N 次(JVM 冷启动 + 新建连接 + 加载驱动)**,`from_db.rs` 还是串行 for。真正的元数据查询仅几十毫秒,时间全烧在重复固定开销。
   - **native(MySQL/PG)不受影响**:两者用连接池(`mysql_async::Pool` / `deadpool_postgres::Pool`)跨调用复用连接,非瓶颈。**故批量化只针对 JDBC。**

2. **像没反应 = 按钮无 loading**
   - `ImportWizard.vue` 的「导入」按钮和 `doImport()` 无 `:loading`/禁用,点下去 dialog 干等到整批结束(同文件 step0 按钮反而有 loading,对比明显)。

## Requirements

### A. 批量导入(根治慢)—— 一次 spawn 反解所有选中表
- connector 新增 action `importTables`:入参含选中表名列表,Java 侧**开一条连接**,循环反解每张表的列+索引,一次性返回 `{tables:[{name, columns, indexes}...]}`。
- Rust `Driver` trait 新增批量方法 `import_tables(&self, tables) -> Vec<TableMeta>`:
  - JDBC 实现:一次 `call("importTables", ...)`。
  - native 实现:**默认方法**在 trait 内循环调 `get_columns`/`list_indexes`(池化复用连接,无需各驱动改代码)。
- `import/from_db.rs` 的 `import_from_db` 改为调用 `driver.import_tables(...)` 一次拿全量,删除逐表串行 for。
- 表注释沿用现有 `TableInfo.comment`(listTables 已带),不额外查。

### C. 导入按钮 loading 态(止血像没反应)
- `doImport` 期间「导入」按钮 `:loading` + 禁用,期间禁止关闭 dialog。
- 单个 spinner 即可,**不做**流式逐表进度条(NDJSON emit):A 落地后单次等待已降至秒级,进度条属过度设计,除非未来实测表数上百仍嫌黑箱再评估。

## 非目标(明确不做)
- 不上常驻 Java daemon(与 architecture.md §2「一次性命令」决策冲突,批量化已吃掉其大部分收益)。
- 不动 native 驱动的连接/查询逻辑(非瓶颈)。
- 不做流式进度条 / 不改导入向导的步骤结构。

## Acceptance Criteria
- [ ] connector 新增 `importTables` action,一条连接反解多表,契约对齐 `subprocess-io-contract.md`(UTF-8/错误协议)。
- [ ] `Driver::import_tables` 批量方法落地:JDBC 一次 spawn;native 走 trait 默认实现(池化,无回归)。
- [ ] `import_from_db` 改用批量方法,JDBC 导 N 表 spawn 次数从 `2N` 降到 `1`。
- [ ] 导入按钮在 `doImport` 期间显示 loading 且禁止关闭 dialog。
- [ ] `cargo test`(aqua-core)+ connector 单测通过;JDBC 手工验证导多表结果与旧逻辑一致(列/索引/注释无缺失)。
