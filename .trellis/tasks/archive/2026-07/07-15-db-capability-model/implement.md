# 数据库清单简化与反解通用化 - 实施计划

## 实施顺序(按 design.md 的 10 步)

### 步骤 1: Rust 侧清单删减 [完成]
**文件**: `crates/aqua-core/src/driver/dialects.rs`

- [x] 删除 `ALL_DATABASES` 中的 TiDB / GaussDB / OceanBase 三个条目
- [x] 更新注释:从"11 个"改"8 个"
- [x] DM/KingBase/GBase/SQLServer 的 `reverse_supported` 从 `false` 改 `true`,注释标记"通用兜底反解,未实测"
- [x] 测试调整:
  - `test_list_dialects_count`: `assert_eq!(len, 11)` → `assert_eq!(len, 8)`
  - `test_reverse_supported_set`: 期望改为全部 8 个库
  - `test_builtin_driver_set`: 保持 3 个(mysql/pg/h2)

**验证**: `cargo test -p aqua-core --lib driver::dialects::tests`

---

### 步骤 2: Rust 侧状态简化 [完成]
**文件**: `crates/aqua-core/src/driver/state.rs`

- [x] `DatabaseState` 结构删除 `hidden` 字段
- [x] 删除 `set_hidden` 函数
- [x] `list_databases_with_state` 中 `hidden` 固定返回 `false`(前端类型兼容)
- [x] 测试调整:
  - `test_state_roundtrip`: 删除 hidden 初始化和断言
  - `test_load_missing_returns_default`: 删除 hidden 断言
  - `test_list_databases_with_state`: 清单数改 8,删除 hidden 相关断言
  - `test_set_hidden_toggle`: 整个删除

**验证**: `cargo test -p aqua-core --lib driver::state::tests`

---

### 步骤 3: Rust 侧 command 删除 [完成]
**文件**: 
- `src-tauri/src/commands/database.rs`
- `src-tauri/src/lib.rs`

- [x] `database.rs`: 删除 `set_database_hidden` 函数
- [x] `lib.rs`: 从 `invoke_handler!` 宏中删除 `database::set_database_hidden`

**验证**: `cargo clippy --all-targets`

---

### 步骤 4: connector 抽基类 [完成]
**文件**: `connector/src/main/java/com/aqua/connector/AbstractJdbcDialect.java` (新建)

- [x] 创建抽象类 `AbstractJdbcDialect implements Dialect`
- [x] 实现 `connect` 方法(driver.connect 通用加载逻辑)
- [x] 实现 `listTables` 方法(标准 JDBC `DatabaseMetaData.getTables` 遍历)
- [x] 实现 `getColumns` 方法(标准 JDBC `getColumns` + `getPrimaryKeys` 遍历)
- [x] 实现 `getIndexes` 方法(标准 JDBC `getIndexInfo` 遍历 + 跳过主键索引)
- [x] 定义抽象方法:
  - `getDriverClass()`: 驱动类名
  - `buildUrl(DbConfig)`: 构造 JDBC URL
  - `mapType(jdbcType, typeName, precision, scale)`: 类型映射
  - `resolveSchema(Connection, schema)`: schema 解析钩子(默认直接返回 schema)

---

### 步骤 5: connector 通用兜底 [完成]
**文件**: `connector/src/main/java/com/aqua/connector/GenericJdbcDialect.java` (新建)

- [x] 创建 `GenericJdbcDialect extends AbstractJdbcDialect`
- [x] 构造函数接受 `dialectName` 和 `driverClass`
- [x] `buildUrl` 实现:`jdbc:<dialectName>://<host>:<port>/<database>`
- [x] `mapType` 实现:按 `java.sql.Types` 标准映射(TINYINT/INT/LONG/DECIMAL/VARCHAR/CLOB/BLOB/DATE/DATETIME 等)
- [x] 未知类型兜底为 `VARCHAR`

---

### 步骤 6: connector 重构现有实现 [完成]
**文件**: 
- `connector/src/main/java/com/aqua/connector/h2/H2Dialect.java`
- `connector/src/main/java/com/aqua/connector/oracle/OracleDialect.java`

- [x] H2Dialect 改为 `extends AbstractJdbcDialect`
  - 删除 `listTables/getColumns/getIndexes/connect` 重复代码
  - 保留 `buildUrl`(H2 特殊 URL:mem/file/tcp)
  - 保留 `mapType`(调用 H2TypeMapping)
- [x] OracleDialect 改为 `extends AbstractJdbcDialect`
  - 删除重复代码
  - 保留 `buildUrl`(jdbc:oracle:thin:@//...)
  - 保留 `mapType`(调用 OracleTypeMapping,NUMBER 按 precision/scale 反解)
  - 覆写 `resolveSchema`(返回 `conn.getSchema()`,Oracle schema=用户名)

**验证**: `mvn test` (H2DialectTest 行为不变)

---

### 步骤 7: connector 注册兜底 [完成]
**文件**: `connector/src/main/java/com/aqua/connector/DialectRegistry.java`

- [x] static 块新增注册:
  ```java
  register(new GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver"));
  register(new GenericJdbcDialect("kingbase", "com.kingbase8.Driver"));
  register(new GenericJdbcDialect("gbase", "com.gbase.jdbc.Driver"));
  register(new GenericJdbcDialect("sqlserver", "com.microsoft.sqlserver.jdbc.SQLServerDriver"));
  ```

**验证**: `mvn test`

---

### 步骤 8: 前端 UI 删减 [完成]
**文件**: `app/src/components/DatabaseConfigDialog.vue`

- [x] 删除"显示"列(`<el-table-column label="显示">` 及 el-switch)
- [x] 删除 `onToggleVisible` 函数
- [x] 提示文案改为"未装驱动的外置库不出现在下拉中"

---

### 步骤 9: 前端过滤逻辑 [完成]
**文件**: 
- `app/src/stores/database.ts`
- `app/src/composables/useTauri.ts`

- [x] `database.ts`:
  - `generatable` 过滤改为 `d.builtinDriver || d.installed`
  - `reversible` 过滤改为 `(d.builtinDriver || d.installed) && d.reverseSupported`
- [x] `useTauri.ts`: 删除 `setDatabaseHidden` 方法

**验证**: `pnpm vue-tsc --noEmit`

---

### 步骤 10: 全面验证 [完成]

- [x] `cargo fmt --all`
- [x] `cargo clippy --all-targets` → 无 warning
- [x] `cargo test -p aqua-core --lib` → 53 passed
- [x] `mvn test` → 3 passed, BUILD SUCCESS
- [x] `pnpm vue-tsc --noEmit` → 无类型错误

---

## 手动验证清单(待用户测试)

需在实际运行环境验证:

- [ ] 启动 app,打开"数据库配置"弹窗:
  - [ ] 不再有"显示"开关列
  - [ ] 只有"数据库 / 类型 / 驱动 / 操作"四列
- [ ] 未装驱动的外置库(Oracle/SQLServer/DM/KingBase/GBase):
  - [ ] 不出现在"生成 DDL"下拉
  - [ ] 不出现在"从数据库导入"反解下拉
  - [ ] 不出现在"数据源配置"类型下拉
- [ ] 安装一个外置库驱动(如 Oracle ojdbc11.jar):
  - [ ] 安装后,Oracle 出现在三处下拉
  - [ ] 测试连接可用
  - [ ] 反解表结构可用(类型映射为 Oracle 专门实现或通用兜底)
- [ ] 内置库(MySQL/PostgreSQL/H2):
  - [ ] 始终在三处下拉中可见
  - [ ] 无"安装"按钮(驱动列显示"内置")

---

## 回滚计划

如需回滚:

1. **清单恢复**: `git checkout HEAD -- crates/aqua-core/src/driver/dialects.rs`,恢复 TiDB/GaussDB/OceanBase 三条目(但连接/反解仍是断的,只恢复生成 DDL 能力)
2. **hidden 恢复**: 回滚 state.rs/commands/UI,但需重新设计隐藏配置价值(PRD 已明确无价值)
3. **connector 回滚**: 删除 AbstractJdbcDialect/GenericJdbcDialect,恢复 H2/Oracle 旧实现。信创库反解能力丢失。

回滚成本高,且 PRD 验收的问题(冗余清单、hidden 多余、反解重复)会重现。建议只在发现严重 bug 时回滚,然后修复后重新合入。

---

## 依赖与风险

- **无外部依赖**:本任务不依赖其他任务,可独立合入。
- **风险点**:
  - GenericJdbcDialect 的类型映射未在 DM/KingBase/GBase/SQLServer 真实环境验证,可能有类型反解偏差(如 NUMERIC 精度丢失)。已标记"未实测",后续有环境再校准。
  - 删除 TiDB/GaussDB/OceanBase 后,旧项目文件若在生成选项里引用这些库名,会失败(但这些库原本就连不上、不能反解,只能生成 DDL,影响面极小)。
