# 数据源 JDBC URL 配置模式

## Goal

为 JDBC 类数据源提供"URL 直填"配置模式,解决"主机+端口+库名"三元组无法表达部分连接形态的问题(首要痛点:H2 文件库)。用户已选定方案 A:URL 模式仅对 Jdbc 类 dialect 开放,原生 mysql/pg 不受影响。

## Background(已核实事实)

### H2 已在链路里,但只能连 mem/tcp

- H2 已注册为完整 dialect:Rust 清单 `crates/aqua-core/src/driver/dialects.rs:67-77`(`builtin_driver: true`,驱动 shade 进 connector.jar)、Java 注册 `connector/src/main/java/com/aqua/connector/DialectRegistry.java:20`。
- `H2Dialect.buildUrl()`(`connector/src/main/java/com/aqua/connector/h2/H2Dialect.java:29-34`)只有两条分支:host 为空或 `mem` → `jdbc:h2:mem:<db>;DB_CLOSE_DELAY=-1`;否则 → `jdbc:h2:tcp://<host>:<port>/<db>`。注释里写了 `jdbc:h2:file:<path>` 但未实现——桌面场景最常用的 H2 文件库无法配置,这正是本次要解的问题。
- 相关 bug:Rust 侧 H2 默认端口 `8082`(`dialects.rs:71`)是 web console 端口,tcp server 实为 9092(前端硬编码表 `app/src/views/config/DataSourcePanel.vue:32` 写的 9092 才对)。

### 当前是纯固定字段模型,无 URL 逃生舱

- 全仓库检索 `jdbc_url` / `jdbcUrl` / `customUrl` / `extra_params` / "高级模式" 等,零命中。
- 连接字段在三个孪生类型上手工同步(新增字段需同步 5 处):
  - `DataSourceConfig`(持久化态)`crates/aqua-core/src/datasource/mod.rs:32-41`
  - `DbConfig`(连接态)`crates/aqua-core/src/driver/types.rs:8-23`
  - TS `DbConfig` `app/src/types/schema.ts:136-144`、`DataSource` `app/src/stores/datasource.ts:8-11`(wire 格式 `Array<[string, DbConfig]>`,拆装 `stores/datasource.ts:14-22`)
  - Java `DbConfig` `connector/src/main/java/com/aqua/connector/DbConfig.java:13-27`(`@JsonIgnoreProperties(ignoreUnknown = true)`)

### URL 拼接位置与路由

- native(mysql/postgres)不拼 URL,builder 逐字段传参:`crates/aqua-core/src/driver/mysql.rs:19-33`、`postgres.rs:18-36`。本次不动 native 链路。
- JDBC 侧 URL 在 Java 拼:Rust 只透传字段 JSON(`crates/aqua-core/src/driver/jdbc.rs:284-307`),`AbstractJdbcDialect.connect()`(`connector/src/main/java/com/aqua/connector/AbstractJdbcDialect.java:58-75`,final)调用子类 `buildUrl`,user/password 走 `Properties` 不进 URL。
- dialect 路由:`crates/aqua-core/src/driver/factory.rs:21-36`,未知 dialect 兜底走 JDBC。
- `DatabaseInfo.category`(`crates/aqua-core/src/driver/state.rs:30-44`,camelCase 序列化)已把 `"native" | "jdbc"` 传到前端(`app/src/types/schema.ts:147-160`),URL 模式可按 category 门控。

### 前端表单现状

- `app/src/views/config/DataSourcePanel.vue`:表单 model `:16-24`,类型下拉来自 `dbStore.reversible`;端口默认值是硬编码 `DEFAULT_PORTS` 表(`:27-38`),与 Rust `defaultPort` 重复且含死配置(见验收标准 AC5)。

### 相关 spec(实现前必读)

- `.trellis/spec/connector/backend/dialect-extension.md` —— Dialect 契约、三钩子模板方法、Rust↔Java 清单同步规则(`:227-256`)
- `.trellis/spec/connector/backend/subprocess-io-contract.md` —— Rust↔Java stdin/stdout 契约
- `.trellis/spec/aqua-core/backend/database-guidelines.md` —— Rust Driver trait 原则
- `.trellis/spec/app/frontend/type-safety.md`、`.trellis/spec/app/frontend/state-management.md`

## Requirements

### R1:URL 模式数据模型与透传(全链路)

- `DataSourceConfig` / `DbConfig`(Rust)、`DbConfig`(TS)、`DbConfig`(Java)四处新增可选字段 `jdbcUrl`(serde/驼峰命名对齐现有字段风格)。
- 旧配置文件(无该字段)反序列化必须兼容,`jdbcUrl` 为 `None` 时行为与现在完全一致。
- JDBC 请求基础 payload(`jdbc.rs:284-307`)透传 `jdbcUrl`(非空时)。

### R2:Java 侧 URL 直连

- `AbstractJdbcDialect.connect()`:`config.jdbcUrl` 非空时直接用它作为 JDBC URL,跳过 `buildUrl()`;user/password 仍走 `Properties`。
- 顺带实现 `H2Dialect` 注释中承诺但未实现的 `jdbc:h2:file:<path>` 分支(host 填 `file` 时)。

### R3:前端 URL 模式表单

- `DataSourcePanel.vue`:当所选 dialect 的 `category === "jdbc"` 时,提供"主机+端口 / URL"两种配置方式切换(默认主机+端口,保持现有习惯)。
- URL 模式下表单只展示:数据源名称、类型、JDBC URL、用户名、密码;host/port/database 不展示、不校验。
- 切换回主机+端口模式时保留此前填写的三元组值(不丢失)。
- 测试连接、保存、编辑既有数据源在两种模式下均正常工作;URL 模式保存后再次编辑能还原为 URL 模式。

### R4:H2 默认端口修正

- Rust `dialects.rs` 中 H2 `default_port` 由 8082 改为 9092(web console 端口 → tcp server 端口),使默认值接 tcp URL 能连通。

### R5:前端端口默认值去重

- `DataSourcePanel.vue` 的硬编码 `DEFAULT_PORTS` 改为从 `dbStore.databases` 的 `defaultPort` 取值,消除双份维护(顺带清理 gaussdb/oceanbase 死配置、补 gbase)。

## Acceptance Criteria

- [ ] AC1:对一个 JDBC 类 dialect(如 H2)以 URL 模式配置 `jdbc:h2:file:<路径>` + 任意用户名密码,点"测试连接"成功。
- [ ] AC2:以 URL 模式配置 `jdbc:h2:mem:<名>;DB_CLOSE_DELAY=-1`,保存 → 重启应用(重新 load)→ 编辑该数据源,表单还原为 URL 模式且 URL 值完整;密码仍为密文持久化(检查 `.conf` 文件中无明文密码)。
- [ ] AC3:旧格式数据源文件(无 `jdbcUrl` 字段)加载后行为不变,主机+端口模式正常连接 mysql(回归)。
- [ ] AC4:选择 mysql/postgresql(native 类)时,表单不出现"URL 模式"切换入口。
- [ ] AC5:H2 dialect 选中时默认端口显示 9092(来自后端 `defaultPort`,前端无硬编码表)。
- [ ] AC6:`cargo test -p aqua-core`、connector 单测(`mvn test` 或现有命令)、前端 `pnpm typecheck`(或等价命令)全部通过。

## Out of Scope

- native(mysql/postgresql)驱动不支持自定义 URL 方言(方案 A 明确排除)。
- `schema` 输入框缺失、JDBC 基础 payload 不带 schema 等既有半悬空问题(独立问题,不在本任务修)。
- URL 内嵌用户名/密码的解析(URL 模式下凭据仍走独立字段 + `Properties`)。
- 表单整体校验体系改造(仅做 URL 模式所需的最小校验:URL 非空)。

## Key Decisions

- **D1(用户已定)**:URL 模式仅对 Jdbc 类 dialect 开放,前端按 `DatabaseInfo.category === "jdbc"` 门控;native 走原有 builder 路径不变。
- **D2**:字段命名统一 `jdbcUrl`(Rust `jdbc_url` + serde 默认驼峰,与现有 wire 对齐)。
- **D3**:URL 模式是"字段级可选"而非新数据源类型——同一 `DataSourceConfig` 靠 `jdbcUrl` 是否非空区分,持久化格式向后兼容。
- **D4**:不引入 URL 语法校验,交给驱动报错透传(错误信息本就经 connector stdout 协议回传)。
