# Design: 数据库支持与驱动管理系统

## 架构边界

| 层 | 位置 | 职责 |
|---|---|---|
| 生成面 | `crates/aqua-core/src/generators/ddl/` (Rust) | `map_type` + 数据库清单,零 java |
| 反解面 | `connector/` (Java) | `Dialect` 接口 + 各数据库实现 + drivers/ 加载 |
| 命令层 | `src-tauri/src/commands/` | 数据库清单查询、databases.json 读写、驱动装/卸 |
| 前端 | `app/src/` | 数据库配置页 + 三处下拉 |

> 代码层 `Dialect` 类型名保留(SQL 方言行业术语);用户可见层用"数据库"。

## 数据流

**生成**(零 java):
```
Project + Dialect -> generate_ddl -> Rust map_type(硬编码/复用) -> DDL 文本
```

**反解**(spawn java):
```
DbConfig -> JdbcDriver.call -> spawn connector.jar
  -> Main 读 databases.json, URLClassLoader 加载 driverJar
  -> Dialect.connect + listTables/getColumns/getIndexes
  -> ColumnMeta/IndexMeta(JSON) -> Rust 解析
```

**列表**(零 java):
```
Rust 静态数据库清单 + databases.json(hidden/installed)
  -> Tauri command list_databases()
  -> 前端按场景过滤
```

## 数据库清单结构(Rust)

```rust
struct DialectInfo {
    name: &str,           // "oracle"
    label: &str,          // "Oracle"
    category: DialectCat, // Native | Jdbc
    default_port: u16,
    needs_schema: bool,
    generate_as: Option<&str>,  // 兼容层复用: "mysql"/"postgresql"
    driver_class: Option<&str>, // JDBC Driver 全限定类名
    reverse_supported: bool,    // 是否有反解 Dialect 实现
    builtin_driver: bool,       // 驱动是否内置(native/h2)
}
```

清单(11 个):

| name | category | port | schema | generateAs | driverClass | reverse | builtin |
|---|---|---|---|---|---|---|---|
| mysql | Native | 3306 | f | - | - | ✓ | ✓ |
| postgresql | Native | 5432 | t | - | - | ✓ | ✓ |
| h2 | Jdbc | 8082 | f | - | org.h2.Driver | ✓ | ✓(shade) |
| oracle | Jdbc | 1521 | t | - | oracle.jdbc.OracleDriver | ✓(新) | ✗ |
| dm | Jdbc | 5236 | t | - | dm.jdbc.driver.DmDriver | ✗ | ✗ |
| kingbase | Jdbc | 54321 | t | - | com.kingbase8.Driver | ✗ | ✗ |
| gbase | Jdbc | 5258 | t | - | com.gbase.jdbc.Driver | ✗ | ✗ |
| sqlserver | Jdbc | 1433 | f | - | com.microsoft.sqlserver.jdbc.SQLServerDriver | ✗ | ✗ |
| oceanbase | Jdbc | 2881 | t | mysql | - | ✗ | ✗ |
| tidb | Jdbc | 4000 | f | mysql | - | ✗ | ✗ |
| gaussdb | Jdbc | 5432 | t | postgresql | - | ✗ | ✗ |

`reverse_supported`:本任务只 Oracle + H2 为 true。其余 false(生成可用,反解/驱动安装不可用,后续补)。

## databases.json

应用级,路径 `<app_data>/drivers/databases.json`:
- `hidden: string[]` - 隐藏的数据库名
- `installed: [{ dialect, driverJar, driverClass }]` - 已装驱动

字段名 `dialect` 与 `DbConfig.dialect` 一致(代码层,不拆分命名)。connector 启动读此文件,installed 的 jar 加进 URLClassLoader。

## connector 改动

- `Dialect` 接口:保持反解四方法,不加生成面
- `Main`:启动读 databases.json,构造 URLClassLoader(drivers/*.jar),setContextClassLoader
- `OracleDialect`:JDBC 连接 + ALL_TABLES/ALL_TAB_COLUMNS/ALL_IND_COLUMNS 查询 + 物理类型->逻辑类型反解
- `DialectRegistry`:注册 OracleDialect

## Tauri commands

- `list_databases()` -> 返回数据库清单 + 每个的 hidden/installed 状态(合并 Rust 清单 + databases.json)
- `install_driver(dialect, jar_path)` -> copy jar 到 drivers/ + 写 installed
- `uninstall_driver(dialect)` -> 删 jar + 删 installed
- `set_database_hidden(dialect, hidden)` -> 写 hidden

## 三处过滤规则

```
生成下拉:  visible = !hidden
反解下拉:  usable = !hidden && (builtin_driver || installed) && reverse_supported
数据库配置:  全集, 展示 installed + hidden
```

注:反解下拉额外要 `reverse_supported`,否则装了驱动但没 Dialect 实现会失败。

## 权衡

1. **生成 Rust 硬编码 vs connector 提供**:选 Rust。零 java、离线、无 spawn 开销。代价:Rust↔connector 数据库名人工同步(11 个,低频)。
2. **c1 内置 vs c2 外置数据库支持 jar**:选 c1。确定要的库写死够用。ServiceLoader 留口子,等"内置没有的新库"再补。
3. **databases.json 应用级 vs 项目级**:选应用级。驱动装一次全局用,跟项目无关。
4. **反解范围**:生成 11 个全做;反解本任务只 Oracle(+H2 已有)。其余 reverse_supported=false,后续按需补。避免反解系统表查询的巨大工作量阻塞生成+驱动管理。

## 兼容性

- `DbConfig.dialect` 字符串不变
- mysql/pg native 驱动不变
- DataSourceDialog 下拉改动态(替硬编码 `dialects = [...]`)
- 现有 DDL 生成 API(`generate_ddl`/`DdlOptions`)不变,只是 map_type 覆盖全

## 风险

- Oracle 系统表反解需实测(ALL_TAB_COLUMNS 字段、类型映射)
- drivers/ 路径解析:Tauri 打包后 resource/appdata 路径,需确认 connector 能拿到
- URLClassLoader + shade 后的 connector.jar 类加载冲突(ojdbc vs h2)
- ojdbc8 许可证:drivers/ 不预置,文档提示用户自备
