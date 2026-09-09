# 执行计划:数据源 JDBC URL 配置模式

## 顺序清单

按依赖排序:Rust 类型 → Java(可并行) → Rust 透传 → 前端 → 测试收口。

### 1. Rust 数据模型
- [ ] `crates/aqua-core/src/datasource/mod.rs`:`DataSourceConfig` 加 `#[serde(rename_all = "camelCase")]` + `jdbc_url: Option<String>`(带 `skip_serializing_if` / `default`)
- [ ] `crates/aqua-core/src/driver/types.rs`:`DbConfig` 同步加字段
- [ ] 检查两 struct 现有构造点(测试/调用方)是否需要补 `jdbc_url: None`
- [ ] `dialects.rs:71` H2 `default_port` 8082 → 9092,修关联单测断言

### 2. Java connector(改后必须 `pnpm build:connector` 重建 jar)
- [ ] `DbConfig.java` 加 `public String jdbcUrl;`
- [ ] `AbstractJdbcDialect.connect()`:`jdbcUrl` 非空时短路,跳过 `buildUrl()`
- [ ] `H2Dialect.buildUrl()` 补 `file` 分支(`jdbc:h2:file:<database>;AUTO_SERVER=TRUE`)
- [ ] connector 单测:URL 短路逻辑 + H2 file URL 构造

### 3. Rust JDBC 透传
- [ ] `crates/aqua-core/src/driver/jdbc.rs` `build_request`:非空时把 `jdbcUrl` 加进基础 payload
- [ ] aqua-core 单测:payload 含/不含 jdbcUrl 两态

### 4. 前端
- [ ] `app/src/types/schema.ts`:`DbConfig` 加 `jdbcUrl?: string`
- [ ] `app/src/views/config/DataSourcePanel.vue`:
  - `urlMode` 状态 + category 门控的切换控件(native 不显示)
  - URL 模式字段显隐 + jdbcUrl 非空校验
  - `editSource()` 按 `jdbcUrl` 还原模式
  - `onDialectChange()` 切 native 自动退出 URL 模式;默认端口改从 `dbStore.databases` 取,删 `DEFAULT_PORTS`
  - `testConnection()` 传参带 `jdbcUrl`

### 5. 验证收口(AC1–AC6)
- [ ] `pnpm build:connector` 重建 jar
- [ ] `cargo test -p aqua-core`
- [ ] connector 测试(按仓库现有命令,见下)
- [ ] 前端 typecheck / lint(按仓库现有命令)
- [ ] 手工验证:AC1(H2 file URL 测试连接)、AC2(保存重启还原 + 密文检查)、AC3(旧配置回归 mysql)、AC4(native 无切换入口)、AC5(默认端口 9092)

## 验证命令

```bash
pnpm build:connector                 # 改 Java 后必须,否则 dev 跑旧 jar
cargo test -p aqua-core
cd connector && mvn test             # 若仓库无包装脚本,以 package.json scripts 为准
pnpm typecheck                       # 前端,以 package.json 实际脚本名为准
```

## 风险文件与回滚点

- `AbstractJdbcDialect.java` 是所有 JDBC dialect 的公共连接入口——短路分支必须放在 `buildUrl()` 调用处,不触碰 Properties 逻辑;出问题影响全部 JDBC 库连接,回滚即删分支。
- `DataSourceConfig` serde 改动影响已落盘 `.conf` 兼容性——`rename_all` 对现有字段是 no-op,但需跑 `datasource/mod.rs:337-351` 契约测试确认。
- `DataSourcePanel.vue` 表单无 `:rules` 体系,新增校验用命令式判断,不要顺手引入表单校验框架(Out of Scope)。

## 提交策略

单任务单 PR/commit 链:模型(Rust+Java) → 透传 → 前端 → 测试。connector jar 重建产物不入库(以 .gitignore 现状为准)。
