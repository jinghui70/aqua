# Implement: 数据库支持与驱动管理系统

## 阶段 A:Rust 生成侧数据库补全 + 清单

- `generators/ddl/types.rs`:补全 `map_type` 的 oracle/dm/kingbase/gbase/sqlserver 映射;兼容层 oceanbase/tidb/gaussdb 通过 `generateAs` 复用 mysql/postgresql
- 新增数据库清单模块(如 `generators/ddl/dialects.rs` 或 driver 模块):`DialectInfo` + 11 条静态数据 + `list_dialects()`
- `Dialect` 枚举/`map_type` 支持 generateAs 复用
- 单测:每个数据库每种 DataType 的 map_type 输出

验证:`cargo test -p aqua-core`、`cargo fmt`、`cargo clippy`

## 阶段 B:connector Oracle 反解 + drivers/ 加载

- `Dialect` 接口不动
- `Main`:读 databases.json(路径参数化),构造 URLClassLoader(drivers/*.jar),setContextClassLoader
- `OracleDialect`:connect(jdbc:oracle:thin)、listTables(ALL_TABLES)、getColumns(ALL_TAB_COLUMNS + 反解类型)、getIndexes(ALL_IND_COLUMNS)
- `DialectRegistry` 注册 OracleDialect
- 本地用 ojdbc8 + Oracle 实例/容器实测反解

验证:connector 单独跑,stdin JSON 测 listTables/getColumns/listIndexes

## 阶段 C:databases.json 读写 + Tauri commands

- `src-tauri/src/commands/`:新增 database 命令模块
  - `list_databases()`:合并 Rust 清单 + databases.json,返回带 hidden/installed 状态
  - `install_driver`/`uninstall_driver`/`set_database_hidden`
  - databases.json 路径:appdata/drivers/
- drivers/ 目录创建、jar copy
- 前端 useTauri 绑定

验证:`cargo build`、命令手测

## 阶段 D:数据库配置页 UI

- `app/src/views/` 新增 DatabaseConfig 页(或对话框)
- 表格:数据库 | 驱动状态 | 显示开关 | 操作
- 安装:文件选择器 -> 调 install_driver
- 卸载、显隐开关 -> 调对应 command
- native 三禁用卸载
- 路由/入口(设置菜单)

验证:vue-tsc、手测装/卸/开关

## 阶段 E:三处下拉动态化

- 生成目标库下拉:接 list_databases,过滤 !hidden
- 反解数据源下拉(DataSourceDialog 数据库选择):过滤 usable
- 启动加载数据库清单到 store(缓存)
- 移除 DataSourceDialog 硬编码 `dialects = [...]`

验证:vue-tsc、手测三处下拉

## 阶段 F:集成验证

- 无 JDK 环境测 DDL 生成(11 数据库)
- 有 JDK + ojdbc8 测 Oracle 反解
- 数据库配置页装/卸/开关 全流程
- 重启状态保持

验证:`cargo fmt --check && cargo clippy && cargo test`、`pnpm -C app type-check`

## 风险与回滚

- 阶段 B Oracle 反解若系统表查询受阻,不阻塞 A/C/D/E(生成+管理可独立交付),Oracle 反解降级为后续
- drivers/ 路径在打包后需确认,可能需阶段 C 调试
- 每阶段独立提交,便于回滚

## 开始前检查

- [ ] 确认 ojdbc8 获取方式(本地有?需下载?)
- [ ] 确认 drivers/ 在开发态/打包态的路径(resource_dir vs appdata)
- [ ] Oracle 测试库可用性(容器 or 远程)
