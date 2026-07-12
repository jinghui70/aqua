# connector.jar (H2 dialect)

## 背景
aqua v2 JDBC 驱动的 Java 侧实现。Rust 侧 JdbcDriver 通信框架已完成(driver/jdbc.rs),需实现 connector.jar 补全 JDBC 链路。

## 目标
实现 connector.jar Java 项目,支持 H2 dialect,通过 stdin/stdout JSON 与 Rust 通信。

## 包含
- Maven 项目结构(connector/)
- Main 入口:读 stdin JSON,分发 action,写 stdout JSON
- Dialect 接口 + H2Dialect 实现
- JDBC 元数据反解 -> aqua 逻辑类型
- action: testConnection / listTables / getColumns / listIndexes
- Java 单元测试(H2 内存库,JUnit 5)

## 范围(不含)
- Oracle/信创 dialect(后续,机制相同)
- Rust 通信测试(后续任务,需 H2 TCP server)
- H2 TCP server 部署

## 验收标准
- [ ] connector/ Maven 项目(pom.xml + src)
- [ ] Main: stdin JSON -> action 分发 -> stdout JSON
- [ ] H2Dialect: H2 JDBC 元数据 -> aqua 逻辑类型反解
- [ ] 4 个 action 实现并测试
- [ ] Java 单元测试全绿(H2 内存库)
- [ ] mvn package 生成 connector.jar
- [ ] 手动验证: echo JSON | java -jar connector.jar

## 约束
- 兼容 JDK 17+(architecture.md §4)
- H2 驱动打包进 jar(测试用,Oracle 驱动用户自备)
- 错误返回 {"error":"..."} JSON
- 一次性命令,处理完即 exit

## 参考
- Rust 通信协议: crates/aqua-core/src/driver/jdbc.rs
- 架构: docs/architecture.md §7 connector.jar
