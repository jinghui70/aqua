# 数据库清单简化与反解通用化

## 背景与问题

当前数据库清单混入了**协议兼容库**(TiDB/GaussDB/OceanBase),导致几个问题:

1. **冗余**:TiDB/GaussDB 用 MySQL/PG 协议连,用户应直接选 MySQL/PG,不该单列。
   OceanBase 同理(MySQL 模式选 MySQL,Oracle 模式选 Oracle+配 ojdbc jar)。
2. **断裂**:这些库只复用了生成层(`generate_as`),连接层走 JDBC 却 `driver_class: None`,
   **能生成 DDL,不能连、不能反解**。
3. **hidden 配置多余**:显隐本质是"有没有驱动"——内置库永远可见,外置库装了才可见。
   `hidden` 是额外维度,增复杂度却无实际价值。
4. **反解实现重复**:H2Dialect 与 OracleDialect 的 `listTables/getColumns/getIndexes`
   几乎逐行相同,真正因库而异的只有**连接 URL、类型映射、schema 来源**三处。
   若给每个信创库各写一个 Dialect,是大量重复代码。

## Goal

**简化清单 + 反解通用化**:

- 删掉协议兼容库(TiDB/GaussDB/OceanBase),用户用内置 MySQL/PG 代替。
- 清单只保留**内置(MySQL/PG/H2)与独立外置 JDBC 库**(Oracle/DM/KingBase/GBase/SQLServer)。
- 去掉 hidden 配置,外置库未装驱动 → 直接不可见(三处下拉都不出现)。
- 反解通过**基类 + 通用兜底 Dialect** 覆盖所有外置库,类型特例库(如 Oracle)用子类覆写。

## Requirements

### R1 清单简化:删除协议兼容库

- 从 `dialects.rs` 删除 TiDB / GaussDB / OceanBase 三个条目。
- 清单缩减为 8 个:**内置**(mysql/postgresql/h2) + **外置 JDBC**(oracle/dm/kingbase/gbase/sqlserver)。
- 文档/注释/测试中关于这三个库的引用一并清理。

### R2 去掉显隐配置

- 删除 `state.rs` 的 `DatabaseState.hidden` 字段及相关读写逻辑(`set_hidden` 函数)。
- 删除 `commands/database.rs` 的 `set_database_hidden` command。
- 删除前端 `DatabaseConfigDialog.vue` 的"显示"开关列。
- 三处下拉(生成/反解/数据源)过滤逻辑改为:**内置 || 已装驱动**(builtin_driver 或 installed),
  不再检查 hidden。

### R3 反解 Dialect 通用兜底

- connector 侧抽 `AbstractJdbcDialect` 基类,把 `listTables/getColumns/getIndexes` 的标准
  JDBC 遍历实现**一次**,子类只填三个"洞":连接 URL 模板、类型映射函数、schema 解析钩子。
- 提供 `GenericJdbcDialect`:类型映射走 `java.sql.Types` 标准映射,schema 用传入值,
  URL 从 `DbConfig` 直接拼 `jdbc:<dialect>://<host>:<port>/<database>`。
  任何装了 jar 的外置库,无专门实现也能反解(粗粒度逻辑类型)。
- H2Dialect / OracleDialect 重构为 AbstractJdbcDialect 子类,行为不回退
  (Oracle 的 NUMBER 反解、schema=用户名特例保留)。

### R4 反解补齐(兜底或子类)

- 为 DM/KingBase/GBase/SQLServer 开启反解能力:
  - 默认靠 GenericJdbcDialect 兜底(装了 jar 即可反解)。
  - 有类型特例需求的(如达梦),后续按需加子类覆写类型映射,本任务不强求。
- `dialects.rs` 把这四个库的 `reverse_supported` 改为 `true`,注释标记"通用兜底,未实测"。

## 非目标(本任务不做)

- 驱动内置(shade 进 connector.jar):所有外置库(包括 Oracle)保持用户配置 jar 路径,不内置。
- 无环境库的反解**实测校准**:本任务只保证通用兜底代码到位+标记未实测,实测留待有环境时。
- 驱动安装机制调整(copy vs 记路径):保持现状(copy 到 drivers/),不在本任务改动。
- 项目级配置(数据源/项目设置)不动。

## 约束

- 内置驱动固定:**MySQL(native) / PostgreSQL(native) / H2(shade)**,不增不减。
- 外置 JDBC 库:**Oracle / DM / KingBase / GBase / SQL Server**,需用户配置 jar 路径+安装。
- 现有已验证反解(MySQL/PG/H2/Oracle)行为不得回退。
- `dialects.rs` 与 connector `DialectRegistry` 的清单须保持同步(人工或测试校验)。

## Acceptance Criteria

- [ ] `dialects.rs` 删除 TiDB/GaussDB/OceanBase,清单缩减为 8 个,测试/注释一并清理。
- [ ] `state.rs` 删除 `hidden` 字段及 `set_hidden` 函数;command 与 UI 相应删除。
- [ ] 三处下拉(生成/反解/数据源)过滤改为 `builtin || installed`,不再检查 hidden。
- [ ] connector 侧 `AbstractJdbcDialect` + `GenericJdbcDialect` 到位;H2/Oracle 重构为子类,
      行为不回退(Oracle NUMBER/schema 特例保留)。
- [ ] DM/KingBase/GBase/SQLServer 四个库 `reverse_supported: true`,注释标记"通用兜底,未实测"。
- [ ] 数据库配置弹窗不再显示"显示"开关列,驱动安装/卸载列正常工作。
- [ ] 手动验证:未装驱动的外置库不出现在三处下拉;装驱动后出现;native+H2 始终可见。
- [ ] `cargo fmt` / `cargo clippy` / `cargo test` 全绿;`mvn test` 全绿;`vue-tsc` exit 0。

## Notes

- 通用兜底代价:`java.sql.Types` 标准映射得到粗粒度逻辑类型(如所有 NUMERIC→DECIMAL,不像
  Oracle 子类按 precision 反解 TINYINT/INT)。够用,精细化靠后续加子类,不阻塞本任务。
- 用户使用 TiDB/GaussDB/OceanBase 时:选对应协议的内置库(MySQL/PG),或选 Oracle+配 ojdbc jar。
- 删除的三个库在旧版 aqua 中也无连接/反解能力(只能生成 DDL),用户无损。
