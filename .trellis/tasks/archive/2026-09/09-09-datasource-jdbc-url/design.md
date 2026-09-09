# 技术设计:数据源 JDBC URL 配置模式

## 1. 架构与边界

改动横跨四层,但每层改动都很小;核心是"一个可选字段 + 一个连接短路点 + 一个表单切换"。

```
DataSourcePanel.vue          Rust(DataSourceConfig/DbConfig)      Java(AbstractJdbcDialect)
┌────────────────┐   wire    ┌──────────────────────────┐  stdin   ┌─────────────────────┐
│ jdbcUrl?:string│ ────────▶ │ jdbc_url: Option<String> │ ───────▶ │ jdbcUrl: String      │
│ 模式切换(jdbc  │           │ 透传:非空时加进 payload  │          │ connect():非空短路   │
│ category 门控) │           │ 持久化:随 .conf 落盘     │          │ 跳过 buildUrl()      │
└────────────────┘           └──────────────────────────┘          └─────────────────────┘
```

不动的部分:native 驱动(mysql.rs / postgres.rs)、factory 路由、加密持久化逻辑、connector 进程协议结构。

## 2. 数据契约(字段级)

### Rust

`crates/aqua-core/src/datasource/mod.rs:32-41` 与 `crates/aqua-core/src/driver/types.rs:8-23` 两处同步加:

```rust
#[serde(rename_all = "camelCase")]   // 现有字段全是单词,对已落盘数据是 no-op;保证 jdbc_url → jdbcUrl
pub struct ... {
    ...
    /// JDBC URL 直填模式(仅 Jdbc 类 dialect;非空时 Java 侧跳过 buildUrl)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub jdbc_url: Option<String>,
}
```

- 反序列化 `default` 保证旧 `.conf` 与旧前端 payload 兼容(AC3)。
- `skip_serializing_if` 保持落盘文件干净,风格对齐现有 `schema` 字段。

### TS

`app/src/types/schema.ts:136-144` 的 `DbConfig` 加 `jdbcUrl?: string;`。

### Java

`connector/src/main/java/com/aqua/connector/DbConfig.java` 加 `public String jdbcUrl;`(类已 `@JsonIgnoreProperties(ignoreUnknown = true)`,新旧 jar 互不炸)。

## 3. 连接短路点(Java)

`AbstractJdbcDialect.connect()`(`AbstractJdbcDialect.java:58-75`)是 final 模板方法,只在方法体最前面加一个分支,不动钩子体系:

```java
String url = (config.jdbcUrl != null && !config.jdbcUrl.isEmpty())
        ? config.jdbcUrl
        : buildUrl(config);
```

- user/password 仍走 `Properties`(`:67-68`),与现有行为一致——URL 模式不解析 URL 内嵌凭据。
- URL 语法错误由 JDBC 驱动抛 SQLException,经现有 stdout 错误协议(`subprocess-io-contract.md`)回传前端,不做前置校验(PRD D4)。

### H2 file 分支(顺带补齐)

`H2Dialect.buildUrl()`(`h2/H2Dialect.java:29-34`)补 `file` 分支:

```java
if ("file".equalsIgnoreCase(config.host)) {
    return "jdbc:h2:file:" + config.database + ";AUTO_SERVER=TRUE";
}
```

`AUTO_SERVER=TRUE` 允许文件库被多进程并发打开(桌面工具场景常见需求)。注意这是主机+端口模式下的补充;URL 模式下用户直接写完整 URL,不走这里。

## 4. Rust 透传

`crates/aqua-core/src/driver/jdbc.rs:284-307` 的 `build_request`:在基础 payload 后追加

```rust
if let Some(ref url) = self.config.jdbc_url {
    if !url.is_empty() { request["jdbcUrl"] = json!(url); }
}
```

风格对齐现有 `driversDir` 的追加方式(`jdbc.rs:294-296`)。

## 5. 前端(DataSourcePanel.vue)

- **模式状态**:`const urlMode = ref(false)`,当所选 dialect 的 `category === "jdbc"` 时显示 `el-radio-group`(主机+端口 / URL)切换;native 类不渲染切换入口(AC4)。
- **切换逻辑**:切换只改 `urlMode`,不清空 form 中任何字段(三元组与 URL 值并存于 form,落盘时都带上)。
- **字段显隐**:URL 模式下隐藏 host/port/database 输入,显示 `jdbcUrl` 输入框(placeholder 给 `jdbc:h2:file:/path/to/db` 示例);必填校验仅做"URL 模式下 jdbcUrl 非空"。
- **还原逻辑**:`editSource()` 时若 `ds.jdbcUrl` 非空则 `urlMode.value = true`(AC2)。
- **测试连接**:`testConnection()` 传参时带上 `jdbcUrl`(与 form 其余字段一起展开即可,新增字段无需特判)。
- **dialect 切换联动**:`onDialectChange()` 切到 native 类时若处于 URL 模式,自动切回主机+端口模式。
- **默认端口去重(R5)**:删除硬编码 `DEFAULT_PORTS`,`onDialectChange()` 从 `dbStore.databases` 查 `defaultPort`。注意 dbStore 需已 load——组件挂载时若未 loaded 则触发 `dbStore.load()`(类型下拉本就依赖 reversible,实际已保证)。

## 6. H2 默认端口修正(R4)

`crates/aqua-core/src/driver/dialects.rs:71`:`default_port: 8082` → `9092`。同步更新 `dialects.rs` 中断言内置清单的单测(若端口参与断言)与 `connector` 侧 H2 相关测试期望。

## 7. 兼容性

| 面 | 影响 | 结论 |
|---|---|---|
| 旧 `.conf` 文件 | 无 `jdbcUrl` 字段 → `None` | 完全兼容(AC3) |
| 旧前端 payload → 新 jar | 无 `jdbcUrl` → null → 走 buildUrl | 兼容 |
| 新 payload → 旧 jar | 多余字段被 `ignoreUnknown` 忽略,URL 模式静默失效 | 可接受(桌面应用整体发版,不存在长期混跑) |
| `.conf` 密文 | jdbcUrl 不含敏感信息,明文落盘 | 与 database 同级,无新增加密需求 |

## 8. 权衡记录

- **字段级可选 vs 新增"URL 型数据源"类型**:选前者。后者要动 dialect 枚举语义和全部调用方,收益为零。
- **`rename_all = "camelCase"` vs 单字段 `rename`**:选前者,对已落盘数据是 no-op(现有字段无多词),且终结"字段名靠单词巧合对齐"的隐患。
- **不校验 URL 格式**:各 JDBC URL 语法由驱动自身裁决,前置正则校验反而会拒绝合法的方言特定参数。测试连接按钮本身就是校验入口。
