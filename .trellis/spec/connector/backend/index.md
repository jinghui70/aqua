# Connector Backend Specifications

connector 是 aqua 的 JDBC 桥接层,负责外置数据库的连接与元数据反解。

## 规范文档

- [Dialect 扩展规范](./dialect-extension.md) - JDBC Dialect 接口的实现契约与扩展模式

## 何时参考

- 新增数据库支持时,决定用 GenericJdbcDialect 还是专门子类
- 发现类型映射偏差需要写专门子类时
- 需要理解 Rust 侧 dialects.rs 与 Java 侧 DialectRegistry 的同步契约时

## 相关规范

- [aqua-core 数据库指南](../aqua-core/backend/database-guidelines.md) - Rust 侧驱动与清单管理
- [跨层数据流指南](../guides/cross-layer-thinking-guide.md) - Rust 与 Java 的契约同步
