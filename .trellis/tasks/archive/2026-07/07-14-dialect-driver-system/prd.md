# 数据库支持与驱动管理系统

## 背景

aqua 当前只支持 MySQL/PostgreSQL(native Rust 驱动)+ H2(connector.jar 内置 JDBC)。Oracle/信创数据库不支持;DDL 生成对 JDBC 数据库(除 H2 示例)返回 `UNKNOWN_{type}`。数据源对话框的数据库下拉是前端硬编码数组,无法扩展。

需要一套完整的数据库支持管理,支持多数据库生成与反解,且外置 JDBC 驱动(用户自备,许可证要求)。

## 核心概念

- **数据库支持 = 反解 + 生成**
- **生成**:纯逻辑映射(逻辑类型->物理类型 + DDL 语法),不需要连库,全 Rust 硬编码,零 java 依赖
- **反解**:连真实库读元数据,走 connector.jar(JDBC),需 JDBC 驱动 jar 在位
- **驱动**:JDBC 驱动 jar,仅反解需要;生成不需要
- **c1**:数据库支持逻辑内置 connector.jar,不引外置(留 ServiceLoader 口子,c2 再补)

> 代码层 `Dialect` 类型名保留(SQL 方言行业术语,精确);用户可见的文件/页面/UI 一律用"数据库"。

## 需求

### N1 数据库支持范围

内部维护一套数据库清单(Rust 硬编码):

- **核心层**:MySQL、PostgreSQL(native)、H2(connector 内置驱动)、Oracle、DM 达梦、KingBase、GBase、SQL Server
- **兼容层**(生成复用):OceanBase(`generateAs=mysql`)、TiDB(`generateAs=mysql`)、GaussDB/openGauss(`generateAs=postgresql`)

### N2 生成侧(Rust)

- `map_type` 补全所有数据库的类型映射,不再返回 `UNKNOWN`
- 兼容层数据库通过 `generateAs` 复用 MySQL/PostgreSQL 映射
- 硬编码数据库清单(name/label/category/defaultPort/needsSchema/generateAs/driverClass/reverseSupported/builtinDriver)

### N3 反解侧(connector)

- 实现 `OracleDialect`(connect/listTables/getColumns/listIndexes,Oracle 系统表)
- drivers/ 目录加载:connector 启动读 `databases.json` 的 installed,driverJar 加进 URLClassLoader
- H2 已有,保持
- 其余数据库(DM/KingBase/GBase/SQL Server/兼容层)反解标 `reverseSupported=false`,本任务不做,后续补

### N4 databases.json(应用级,drivers/ 下)

```json
{
  "hidden": ["gbase", "tidb"],
  "installed": [
    { "dialect": "oracle", "driverJar": "ojdbc8.jar", "driverClass": "oracle.jdbc.OracleDriver" }
  ]
}
```

- `hidden`:用户隐藏的数据库(不进生成/反解下拉)
- `installed`:已装 JDBC 驱动的数据库(driverJar + driverClass)
- 字段名 `dialect` 与 `DbConfig.dialect` 保持一致(代码层,不拆分命名)

### N5 数据库配置页(新 UI)

- 列出全集数据库,展示两列:驱动状态(内置/已装 jar/未装)、显示开关
- native 三(mysql/pg/h2)驱动固定内置,不可卸载;显示开关可关
- 安装:文件选择器选 JDBC 驱动 jar -> copy 到 drivers/ -> 写 `databases.json` installed -> connector 重载
- 卸载:删 jar + 删 installed 记录
- 仅 `reverseSupported=true` 的数据库可装驱动(其余生成可用但无反解)

### N6 三处列表动态化

- **生成目标库下拉**:`!hidden`(全部显示的数据库,不需驱动)
- **反解数据源下拉**:`!hidden && (native || installed) && reverseSupported`
- **数据源对话框数据库下拉**:同反解规则(替掉硬编码数组)
- 数据库配置页本身不受 hidden 影响(永远全集)

## 约束

- 离线桌面工具,不依赖在线服务/下载
- JDBC 驱动 jar 用户自备(Oracle OTN 等许可证),不打包进发行版
- 用户自备 JDK 17+(反解时 spawn connector)
- 生成零 java 依赖(无 JDK 也能生成 DDL)
- Rust 数据库清单 ↔ connector 反解名人工同步(c1 数据库少,可接受)

## 验收标准

- [ ] 11 个数据库 DDL 生成正确,无 `UNKNOWN`
- [ ] Oracle 装 ojdbc8 后反解成功(list_tables/get_columns/list_indexes)
- [ ] 数据库配置页能装/卸载 jar、切换显示开关,持久化到 `databases.json`
- [ ] 三处下拉按规则过滤,重启后状态保持
- [ ] 无 JDK 环境下 DDL 生成正常(不 spawn java)
- [ ] native 三不可卸载
- [ ] cargo fmt/clippy/test + vue-tsc 全绿
