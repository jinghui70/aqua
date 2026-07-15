# 数据库清单简化与反解通用化 - 技术设计

## 架构概览

本任务分三层改动:

1. **Rust 侧清单与状态**(`dialects.rs` + `state.rs`):删减清单、去 hidden 字段。
2. **connector 侧反解通用化**(Java):抽基类、提供兜底 Dialect、重构现有实现。
3. **前端下拉过滤**(Vue):去 hidden UI、改过滤逻辑为 `builtin || installed`。

## 一、Rust 侧:清单与状态简化

### 1.1 `dialects.rs` 清单删减

**删除条目**:从 `ALL_DATABASES` 删除 TiDB / GaussDB / OceanBase 三条。

**变化前后**:
```rust
// 变化前:11 个
[mysql, postgresql, h2, oracle, dm, kingbase, gbase, sqlserver, oceanbase, tidb, gaussdb]

// 变化后:8 个
[mysql, postgresql, h2, oracle, dm, kingbase, gbase, sqlserver]
```

**反解能力更新**:
- DM / KingBase / GBase / SQLServer 的 `reverse_supported` 从 `false` 改 `true`。
- 注释标记"通用兜底反解,未实测"。

**测试调整**:
- `test_list_dialects_count`:从 `assert_eq!(len, 11)` 改 `assert_eq!(len, 8)`。
- `test_reverse_supported_set`:从 `[mysql, postgresql, h2, oracle]` 改为包含全部 8 个。

### 1.2 `state.rs` 去 hidden 字段

**DatabaseState 简化**:
```rust
// 变化前
pub struct DatabaseState {
    pub hidden: Vec<String>,       // 删掉
    pub installed: Vec<InstalledDriver>,
}

// 变化后
pub struct DatabaseState {
    pub installed: Vec<InstalledDriver>,
}
```

**删除函数**:`set_hidden` 整个删除。

**list_databases_with_state 调整**:
```rust
// 变化前
DatabaseInfo {
    ...
    hidden: state.hidden.contains(&d.name.to_string()),
    ...
}

// 变化后
DatabaseInfo {
    ...
    hidden: false,  // 字段保留(前端类型用),固定 false
    ...
}
```
注:DatabaseInfo 的 `hidden` 字段**暂时保留**(前端 TypeScript 类型可能依赖),但固定返回 `false`,
下个任务清理前端类型时再删。

**测试调整**:
- `test_state_roundtrip`:删除 `hidden` 断言。
- `test_set_hidden_toggle`:整个删除。
- `test_list_databases_with_state`:删除 gbase hidden 断言。

### 1.3 `commands/database.rs` 删 command

删除 `set_database_hidden` 整个函数及其在 `lib.rs` 的注册。

## 二、connector 侧:反解通用化

### 2.1 类层次设计

```
Dialect (interface)
  ├─ AbstractJdbcDialect (abstract class)
  │    ├─ 实现 listTables/getColumns/getIndexes (final,标准 JDBC 遍历)
  │    ├─ 实现 connect (final,driver.connect 通用加载逻辑)
  │    └─ 抽象方法(子类填洞):
  │         - buildUrl(DbConfig): String
  │         - mapType(jdbcType, typeName, precision, scale): DataType
  │         - resolveSchema(Connection, schema): String (默认返回 schema)
  ├─ GenericJdbcDialect extends AbstractJdbcDialect
  │    └─ 零特化:URL 拼 jdbc:<dialect>://..., 类型走 java.sql.Types 标准映射
  ├─ H2Dialect extends AbstractJdbcDialect
  │    └─ 覆写 buildUrl(H2 特殊 URL), mapType(H2TypeMapping)
  └─ OracleDialect extends AbstractJdbcDialect
       └─ 覆写 buildUrl, mapType(OracleTypeMapping), resolveSchema(用 conn.getSchema())
```

### 2.2 AbstractJdbcDialect 实现要点

**connect 实现**(driver.connect 通用加载,复用 OracleDialect 现有逻辑):
```java
public Connection connect(DbConfig config) throws SQLException {
    String url = buildUrl(config);
    try {
        ClassLoader cl = Thread.currentThread().getContextClassLoader();
        if (cl == null) cl = getClass().getClassLoader();
        Driver driver = (Driver) Class.forName(getDriverClass(), true, cl)
            .getDeclaredConstructor().newInstance();
        Properties props = new Properties();
        props.setProperty("user", config.user);
        props.setProperty("password", config.password);
        return driver.connect(url, props);
    } catch (SQLException e) { throw e; }
    catch (Exception e) {
        throw new SQLException("加载驱动失败: " + e.getMessage(), e);
    }
}

protected abstract String getDriverClass();  // 子类返回 driverClass,GenericJdbcDialect 从 name 推断
```

**listTables 实现**(从 H2Dialect 提取):
```java
public final List<String> listTables(Connection conn, String schema) throws SQLException {
    DatabaseMetaData meta = conn.getMetaData();
    String resolvedSchema = resolveSchema(conn, schema);
    List<String> tables = new ArrayList<>();
    try (ResultSet rs = meta.getTables(conn.getCatalog(), resolvedSchema, "%", new String[]{"TABLE"})) {
        while (rs.next()) {
            tables.add(rs.getString("TABLE_NAME"));
        }
    }
    return tables;
}

protected String resolveSchema(Connection conn, String schema) throws SQLException {
    return schema;  // 默认直接用传入值,Oracle 子类覆写为 conn.getSchema()
}
```

**getColumns/getIndexes**:同样从 H2Dialect 提取为 final 方法,调 `mapType(jdbcType, typeName, precision, scale)`。

### 2.3 GenericJdbcDialect 兜底实现

```java
public class GenericJdbcDialect extends AbstractJdbcDialect {
    private final String dialectName;
    
    public GenericJdbcDialect(String dialectName) {
        this.dialectName = dialectName;
    }
    
    @Override public String name() { return dialectName; }
    
    @Override protected String getDriverClass() {
        // 从 name 推断常见 driverClass,或要求在 DialectRegistry.register 时传入
        // 简化:要求调用方提供,或从 databases.json installed 读取
        throw new UnsupportedOperationException("GenericJdbcDialect 需显式 driverClass");
    }
    
    @Override protected String buildUrl(DbConfig config) {
        return String.format("jdbc:%s://%s:%d/%s", dialectName, config.host, config.port, config.database);
    }
    
    @Override protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        // 按 java.sql.Types 标准映射(保守)
        switch (jdbcType) {
            case Types.TINYINT: return DataType.TINYINT;
            case Types.SMALLINT: return DataType.SMALLINT;
            case Types.INTEGER: return DataType.INT;
            case Types.BIGINT: return DataType.LONG;
            case Types.DECIMAL:
            case Types.NUMERIC: return DataType.DECIMAL;
            case Types.FLOAT:
            case Types.REAL: return DataType.FLOAT;
            case Types.DOUBLE: return DataType.DOUBLE;
            case Types.VARCHAR:
            case Types.CHAR:
            case Types.LONGVARCHAR: return DataType.STRING;
            case Types.CLOB: return DataType.TEXT;
            case Types.BLOB:
            case Types.BINARY:
            case Types.VARBINARY:
            case Types.LONGVARBINARY: return DataType.BYTES;
            case Types.DATE: return DataType.DATE;
            case Types.TIME: return DataType.TIME;
            case Types.TIMESTAMP: return DataType.DATETIME;
            case Types.BOOLEAN:
            case Types.BIT: return DataType.BOOLEAN;
            default: return DataType.STRING;  // 未知类型兜底
        }
    }
}
```

### 2.4 H2Dialect / OracleDialect 重构为子类

保持现有行为,只是把重复代码上提到基类:
- `listTables/getColumns/getIndexes` 删掉(基类实现)。
- `connect` 简化为调 `buildUrl` + 基类通用加载(H2 用 DriverManager 可保留特例,或统一走 driver.connect)。
- 保留特化:`mapType` 子类覆写(H2TypeMapping / OracleTypeMapping),Oracle 的 `resolveSchema` 覆写。

### 2.5 DialectRegistry 注册通用兜底

```java
static {
    register(new H2Dialect());
    register(new OracleDialect());
    // 为 DM/KingBase/GBase/SQLServer 注册通用兜底(需知道 driverClass)
    // 选项 1:硬编码注册 GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver")
    // 选项 2:动态从 databases.json installed 读取后注册
    // 本设计选**选项 1**(简单),driverClass 从 dialects.rs 同步过来
}
```

## 三、前端:去 hidden UI + 改过滤

### 3.1 DatabaseConfigDialog.vue

删除"显示"列:
```vue
<!-- 删除这一列 -->
<el-table-column label="显示" width="80" align="center">
  <template #default="{ row }">
    <el-switch :model-value="!row.hidden" @change="..."/>
  </template>
</el-table-column>

<!-- 删除 onToggleVisible 函数 -->
```

### 3.2 stores/database.ts 下拉过滤

```typescript
// 变化前
const generatable = computed(() => databases.value.filter((d) => !d.hidden));
const reversible = computed(() => 
  databases.value.filter((d) => !d.hidden && (d.builtinDriver || d.installed) && d.reverseSupported)
);

// 变化后
const generatable = computed(() => 
  databases.value.filter((d) => d.builtinDriver || d.installed)
);
const reversible = computed(() => 
  databases.value.filter((d) => (d.builtinDriver || d.installed) && d.reverseSupported)
);
```

### 3.3 DataSourceDialog.vue 类型下拉

```vue
<!-- 变化前 -->
<el-option v-for="d in dbStore.reversible" :key="d.name" :label="d.label" :value="d.name" />

<!-- 变化后:同样吃 reversible,逻辑已在 store 改 -->
```

## 四、实现顺序

1. **Rust 侧清单**:删 dialects.rs 三条目、改 reverse_supported、改测试(5 分钟)。
2. **Rust 侧状态**:state.rs 删 hidden 字段/函数、改测试(10 分钟)。
3. **Rust 侧 command**:database.rs 删 set_database_hidden、lib.rs 删注册(2 分钟)。
4. **connector 基类**:抽 AbstractJdbcDialect、实现 listTables/getColumns/getIndexes/connect(30 分钟)。
5. **connector 兜底**:GenericJdbcDialect + 类型映射(15 分钟)。
6. **connector 重构**:H2Dialect/OracleDialect 改继承基类、删重复代码(20 分钟)。
7. **connector 注册**:DialectRegistry 注册 DM/KingBase/GBase/SQLServer 通用兜底(10 分钟)。
8. **前端 UI**:DatabaseConfigDialog 删列、删函数(5 分钟)。
9. **前端 store**:database.ts 改过滤逻辑(3 分钟)。
10. **验证**:cargo/mvn/vue-tsc + 手动测三处下拉(15 分钟)。

总计约 2 小时。

## 五、风险与边界

### 风险 1:GenericJdbcDialect 的 driverClass 来源

问题:通用兜底需知道 driverClass 才能加载驱动,但 `databases.json` 的 `installed[].driverClass` 
是用户装驱动时从 `dialects.rs` 抄过去的——如果 Rust 侧不声明,connector 侧拿不到。

解决:**在 dialects.rs 保留 `driver_class` 字段**(虽不再用于路由判断,但作为元数据供安装流程写入)。
connector 侧从 Main.loadDrivers 读到的 `installed[]` 里就有 driverClass,用它实例化 GenericJdbcDialect。

### 风险 2:H2 的 DriverManager vs driver.connect

H2Dialect 当前用 `DriverManager.getConnection`(H2 驱动 shade 进 jar,在系统类加载器里可见),
改用 driver.connect 是否兼容?

解决:兼容。driver.connect 是更通用的方式,H2 驱动一样支持。统一走 AbstractJdbcDialect.connect 无问题。

### 边界:前端类型 DatabaseInfo.hidden

前端 TypeScript `types/schema.ts` 的 `DatabaseInfo` 可能有 `hidden: boolean` 字段。
本任务 Rust 侧 `DatabaseInfo` 保留该字段(固定返回 false),前端类型暂不动,避免连锁改动。
下个任务或清理时再删。

## 六、测试策略

### 自动化测试
- `cargo test`:dialects.rs 的清单数量、reverse_supported 集合;state.rs 的 roundtrip 不含 hidden。
- `mvn test`:H2DialectTest 跑通(继承基类后行为不变)。
- `vue-tsc`:无类型错误。

### 手动验证
1. **未装驱动的外置库不可见**:启动 app,打开数据库配置,Oracle/SQLServer 等未装驱动 → 三处下拉都不出现。
2. **装驱动后可见**:安装 Oracle 驱动 → 数据源对话框类型下拉出现 Oracle;导入向导下拉出现 Oracle。
3. **内置库始终可见**:MySQL/PG/H2 始终在下拉里。
4. **通用兜底反解**:装一个信创库(如 DM)的 jar,测试连接 + 反解表结构,类型映射为粗粒度(NUMERIC→DECIMAL)。

## 七、回滚计划

本任务删除了三个清单条目。如果用户依赖 TiDB/GaussDB/OceanBase 的**生成 DDL** 能力(虽连接/反解本来就断的),
回滚方案:恢复这三个条目到 dialects.rs,但 `generate_as` 保持复用,不恢复连接路由(本来就没有)。

隐藏配置删除是单向的,如需恢复需重新设计(但 PRD 已明确隐藏无价值,回滚可能性低)。
