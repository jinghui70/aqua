# drivers/

用户提供的 JDBC 驱动 jar 和 resolver jar 放此目录(connector 用 URLClassLoader 加载)。

## 放什么
- `<db>.jar`:JDBC 驱动(如 `oracle.jar`、`dm.jar`、`kingbase.jar`、`gbase8a.jar`)
- `<db>-resolver.jar`:反解 resolver(实现 connector 的 Resolver 接口,把物理类型反解为 aqua 逻辑类型)

## 声明
`registry.json`(项目根)声明每库的 `driverClass` 与 `resolverClass`。加新库:放 jar + 改 registry.json,connector 本体不动。

## 注意
- MySQL/PG 走 Rust native,**不需要**此目录的驱动。
- 此目录不入 git(.gitignore),各用户自备驱动。
