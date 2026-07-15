# Connector - JDBC Dialect 扩展规范

## 概述

connector 是 aqua 的 JDBC 桥接层,通过 `Dialect` 接口封装不同数据库的连接与元数据反解。本文档定义 Dialect 的扩展契约和实现模式。

---

## 核心契约

### Dialect 接口

```java
public interface Dialect {
    String name();  // 方言标识,与 Rust dialects.rs 的 name 字段对应
    Connection connect(DbConfig config) throws SQLException;
    List<String> listTables(Connection conn, String schema) throws SQLException;
    List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException;
    List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException;
}
```

**设计决策**: 反解在 Java 侧(v2 架构),因为 JDBC `DatabaseMetaData` API 是标准且成熟的元数据访问方式,比 Rust 侧用字符串拼 SQL 查系统表更安全、更通用。

---

## 扩展模式

### 模式 1: AbstractJdbcDialect 基类(推荐)

**适用场景**: 标准 JDBC 驱动,元数据遍历走 `DatabaseMetaData` API。

**契约**: 子类只需实现三个钩子:

```java
public abstract class AbstractJdbcDialect implements Dialect {
    // 钩子 1: 驱动类名
    protected abstract String getDriverClass();
    
    // 钩子 2: 构造 JDBC URL
    protected abstract String buildUrl(DbConfig config);
    
    // 钩子 3: 物理类型 -> aqua 逻辑类型映射
    protected abstract DataType mapType(int jdbcType, String typeName, 
                                       Integer precision, Integer scale);
    
    // 可选钩子: schema 解析(默认直接用传入值,Oracle 覆写为 conn.getSchema())
    protected String resolveSchema(Connection conn, String schema) throws SQLException {
        return schema;
    }
}
```

**基类已实现**:
- `connect`: driver.connect 通用加载(支持外置 jar 的 URLClassLoader 隔离)
- `listTables`: `DatabaseMetaData.getTables` 标准遍历
- `getColumns`: `getColumns` + `getPrimaryKeys` 标准遍历
- `getIndexes`: `getIndexInfo` 标准遍历,跳过主键索引

**为什么这三个钩子**: 实测发现,JDBC `DatabaseMetaData` API 在不同数据库间高度一致,真正因库而异的只有:
1. **连接 URL 格式**(如 H2 的 `jdbc:h2:mem:` vs Oracle 的 `jdbc:oracle:thin:@//`)
2. **类型映射**(如 Oracle NUMBER 按 precision 反解为 TINYINT/INT/LONG,H2/MySQL 无此特例)
3. **schema 来源**(大部分库用传入值,Oracle schema=用户名需从连接读)

### Good Case: H2Dialect 子类

```java
public class H2Dialect extends AbstractJdbcDialect {
    @Override
    public String name() { return "h2"; }
    
    @Override
    protected String getDriverClass() { return "org.h2.Driver"; }
    
    @Override
    protected String buildUrl(DbConfig config) {
        if (config.host == null || "mem".equalsIgnoreCase(config.host)) {
            return "jdbc:h2:mem:" + config.database + ";DB_CLOSE_DELAY=-1";
        } else {
            return "jdbc:h2:tcp://" + config.host + ":" + config.port + "/" + config.database;
        }
    }
    
    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        return H2TypeMapping.map(jdbcType, typeName);
    }
}
```

### Good Case: OracleDialect 子类(覆写 resolveSchema)

```java
public class OracleDialect extends AbstractJdbcDialect {
    @Override
    public String name() { return "oracle"; }
    
    @Override
    protected String getDriverClass() { return "oracle.jdbc.OracleDriver"; }
    
    @Override
    protected String buildUrl(DbConfig config) {
        return "jdbc:oracle:thin:@//" + config.host + ":" + config.port + "/" + config.database;
    }
    
    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        return OracleTypeMapping.map(jdbcType, typeName, precision, scale);
    }
    
    @Override
    protected String resolveSchema(Connection conn, String schema) throws SQLException {
        // Oracle schema=登录用户名,不用传入值
        return conn.getSchema();
    }
}
```

---

### 模式 2: GenericJdbcDialect 通用兜底

**适用场景**: 没有类型映射特例的标准 JDBC 库(如 DM/KingBase/GBase/SQLServer)。

**契约**: 构造时传入 `dialectName` 和 `driverClass`,类型映射走 `java.sql.Types` 标准。

```java
public class GenericJdbcDialect extends AbstractJdbcDialect {
    private final String dialectName;
    private final String driverClass;
    
    public GenericJdbcDialect(String dialectName, String driverClass) {
        this.dialectName = dialectName;
        this.driverClass = driverClass;
    }
    
    @Override
    protected String buildUrl(DbConfig config) {
        return String.format("jdbc:%s://%s:%d/%s", 
                            dialectName, config.host, config.port, config.database);
    }
    
    @Override
    protected DataType mapType(int jdbcType, String typeName, Integer precision, Integer scale) {
        // 按 java.sql.Types 标准映射
        switch (jdbcType) {
            case Types.TINYINT: return DataType.TINYINT;
            case Types.SMALLINT:
            case Types.INTEGER: return DataType.INT;
            case Types.BIGINT: return DataType.LONG;
            case Types.DECIMAL:
            case Types.NUMERIC: return DataType.DECIMAL;
            // ... 其他标准映射
            default: return DataType.VARCHAR;  // 未知类型兜底
        }
    }
}
```

**Good Case: 注册到 DialectRegistry**

```java
public class DialectRegistry {
    static {
        register(new H2Dialect());
        register(new OracleDialect());
        // 通用兜底
        register(new GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver"));
        register(new GenericJdbcDialect("kingbase", "com.kingbase8.Driver"));
        register(new GenericJdbcDialect("sqlserver", "com.microsoft.sqlserver.jdbc.SQLServerDriver"));
    }
}
```

**Trade-off**: GenericJdbcDialect 得到**粗粒度逻辑类型**(如所有 NUMERIC→DECIMAL,不像 Oracle 按 precision 反解为 TINYINT/INT/LONG)。够用,精细化靠后续加专门子类,不阻塞反解能力开启。

---

## 决策规则: 何时写专门子类 vs 用通用兜底

| 场景 | 用 GenericJdbcDialect | 写专门子类 |
|------|---------------------|-----------|
| URL 格式标准(`jdbc:<name>://host:port/db`) | ✅ | |
| URL 格式特殊(如 H2 mem/file 模式) | | ✅ |
| 类型映射标准(直接映射 java.sql.Types) | ✅ | |
| 类型映射有特例(如 Oracle NUMBER) | | ✅ |
| schema 用传入值 | ✅ | |
| schema 需特殊解析(如 Oracle=用户名) | | ✅ |
| 无真实测试环境 | ✅ 先兜底 | 有环境再加 |

**原则**: 先用通用兜底开启反解能力,有真实环境验证后发现类型映射偏差再写专门子类。不要为没验证过的库提前写子类。

---

## 与 Rust 侧清单的同步契约

### Rust 侧 (`dialects.rs`)

```rust
pub struct DialectInfo {
    pub name: &'static str,           // 与 Java Dialect.name() 对应
    pub driver_class: Option<&'static str>,  // 提供给安装流程写入 databases.json
    pub reverse_supported: bool,      // true 表示 connector 侧有 Dialect 实现
    // ...
}
```

### Java 侧 (`DialectRegistry`)

```java
static {
    register(new H2Dialect());         // name="h2"
    register(new OracleDialect());     // name="oracle"
    register(new GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver"));  // name="dm"
}
```

### 同步规则

1. **name 字段必须一致**: Rust `DialectInfo.name` == Java `Dialect.name()`。
2. **reverse_supported 声明**: Rust 侧标 `true` 的库,Java 侧必须在 DialectRegistry 注册(专门子类或 GenericJdbcDialect)。
3. **driver_class 保留**: 虽不再用于路由判断,但作为元数据供安装流程写入 `databases.json`,connector 侧从中读取实例化 Dialect。

**验证**: 单测 `dialects.rs` 的 `test_reverse_supported_set` 应断言所有 `reverse_supported=true` 的库,connector 侧都能 `DialectRegistry.get(name)` 拿到非 null 实例(跨语言契约,手动校验)。

---

## 常见错误

### Wrong: 为没验证过的库提前写专门子类

```java
// Wrong: 没有 DM 环境,就写了 DmDialect 子类覆写类型映射
public class DmDialect extends AbstractJdbcDialect {
    @Override
    protected DataType mapType(...) {
        // 凭猜测写的映射,可能是错的
    }
}
```

**问题**: 类型映射错误比通用兜底的粗粒度更糟——反解出来的类型完全不对,用户看到的是错误数据,而通用兜底虽粗但不会错。

### Correct: 先用通用兜底,标记未实测

```java
// Correct: 先注册通用兜底
register(new GenericJdbcDialect("dm", "dm.jdbc.driver.DmDriver"));

// Rust 侧注释标记
DialectInfo {
    name: "dm",
    reverse_supported: true,  // 通用兜底反解,未实测
    // ...
}
```

**验证**: 有 DM 环境后,实测反解类型映射,发现偏差再写 DmDialect 子类覆写。

---

### Wrong: 重复实现 listTables/getColumns/getIndexes

```java
// Wrong: 每个子类都拷贝一遍标准 JDBC 遍历代码
public class MyDialect implements Dialect {
    @Override
    public List<String> listTables(Connection conn, String schema) throws SQLException {
        DatabaseMetaData meta = conn.getMetaData();
        // ... 100 行重复代码
    }
}
```

**问题**: 重复代码难维护,bugfix 需要改 N 处。

### Correct: 继承 AbstractJdbcDialect

```java
// Correct: 只实现三个钩子
public class MyDialect extends AbstractJdbcDialect {
    @Override protected String getDriverClass() { return "..."; }
    @Override protected String buildUrl(DbConfig config) { return "..."; }
    @Override protected DataType mapType(...) { return ...; }
}
```

---

## 扩展清单(当前实现)

| 数据库 | 实现方式 | 状态 |
|--------|---------|------|
| H2 | H2Dialect 子类 | ✅ 已验证 |
| Oracle | OracleDialect 子类 | ✅ 已验证 |
| DM | GenericJdbcDialect | ⚠️ 通用兜底,未实测 |
| KingBase | GenericJdbcDialect | ⚠️ 通用兜底,未实测 |
| GBase | GenericJdbcDialect | ⚠️ 通用兜底,未实测 |
| SQL Server | GenericJdbcDialect | ⚠️ 通用兜底,未实测 |

**未来扩展**: 有真实环境后,按需将通用兜底升级为专门子类。

---

## 相关规范

- [跨层数据流指南](../../guides/cross-layer-thinking-guide.md) - Rust 与 Java 的契约同步思维清单
- [数据库指南](../../aqua-core/backend/database-guidelines.md) - Rust 侧数据库驱动规范
