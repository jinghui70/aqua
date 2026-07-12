# connector.jar 实现计划

## 顺序
1. [ ] pom.xml(Maven + shade + H2 + Jackson + JUnit5)
2. [ ] DataType.java(aqua 逻辑类型枚举)
3. [ ] meta/ColumnMeta.java + IndexMeta.java
4. [ ] DbConfig.java
5. [ ] Dialect.java 接口
6. [ ] h2/H2Dialect.java(JDBC 反解)
7. [ ] DialectRegistry.java
8. [ ] Main.java(stdin -> 分发 -> stdout)
9. [ ] H2DialectTest.java(H2 内存库测试)
10. [ ] mvn package + 手动验证

## 验证
```bash
cd connector
mvn test                    # Java 单元测试
mvn package                 # 生成 connector.jar
echo '{"action":"testConnection","dialect":"h2","database":"mem:test","user":"sa","password":"","host":"","port":0}' | java -jar target/connector.jar
```

## 完成后
- 后续任务: Rust JdbcDriver 通信测试(需 H2 TCP server)
- 后续任务: Oracle dialect(相同机制)
