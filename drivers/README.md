# drivers/

用户提供的 JDBC 驱动 jar 放此目录(connector 用 URLClassLoader 加载)。

> 运行时实际落 `app_data_dir/drivers/`(用户可写);用户经应用内"数据库支持"界面 install_driver 装 jar,自动复制并写入 `databases.json`。项目根此目录仅作开发参考。

## 放什么
- JDBC 驱动 jar(如 `ojdbc8.jar`、`DmJdbcDriver.jar`、`kingbase8.jar`):Oracle/信创等外置库所需。H2 驱动已 shade 进 connector.jar,无需放。

## driverClass 声明
driverClass 硬编码在 `crates/aqua-core/src/driver/dialects.rs` ALL_DATABASES + `connector/.../DialectRegistry.java`(两处人工同步),不经 registry.json。connector 启动读 `databases.json` 的 installed(driverJar + driverClass),用 URLClassLoader 加载。加新库:改两处硬编码 + 用户备 jar,connector 本体不动。

## 注意
- MySQL/PG 走 Rust native,**不需要**此目录的驱动。
- 运行时目录不入 git,各用户自备驱动。
