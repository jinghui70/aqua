# Connector Backend Specifications

connector 是 aqua 的 JDBC 桥接层,负责外置数据库的连接与元数据反解。

## 规范文档

- [Dialect 扩展规范](./dialect-extension.md) - JDBC Dialect 接口的实现契约与扩展模式
- [子进程 IO 契约](./subprocess-io-contract.md) - Rust↔Java 编码/路径/错误协议(UTF-8、strip `\\?\`、stdout 错误、日志)

## 何时参考

- 新增数据库支持时,决定用 GenericJdbcDialect 还是专门子类
- 发现类型映射偏差需要写专门子类时
- 表/列注释缺失时(REMARKS 不可靠,覆写 comment 补查钩子)
- 需要理解 Rust 侧 dialects.rs 与 Java 侧 DialectRegistry 的同步契约时
- 调试连接失败 / 乱码 / Java 打不开 jar / 日志不落盘时(见子进程 IO 契约)

## 相关规范

- [aqua-core 数据库指南](../aqua-core/backend/database-guidelines.md) - Rust 侧驱动与清单管理
- [跨层数据流指南](../guides/cross-layer-thinking-guide.md) - Rust 与 Java 的契约同步
