# connector.jar 技术设计

## 1. 通信协议(对齐 Rust jdbc.rs)

### 请求(stdin JSON)
```json
{
  "action": "listTables",
  "dialect": "h2",
  "host": "localhost", "port": 9092,
  "user": "sa", "password": "",
  "database": "test", "schema": null,
  "table": "SYS_USER"  // getColumns/listIndexes 用
}
```

### 响应(stdout JSON)
- testConnection: `{"status":"ok"}`
- listTables: `{"tables":["SYS_USER","SYS_ROLE"]}`
- getColumns: `{"columns":[{name,dataType,length,precision,scale,nullable,isKey,defaultValue,comment}]}`
- listIndexes: `{"indexes":[{name,fields:[...],unique}]}`
- 错误: `{"error":"..."}`

## 2. 模块结构
```
connector/
  pom.xml
  src/main/java/io/aqua/connector/
    Main.java           # 入口:读 stdin,解析请求,分发,写 stdout
    DbConfig.java       # 连接配置 POJO
    Dialect.java        # 接口
    DialectRegistry.java # dialect name -> Dialect 实例
    DataType.java       # aqua 逻辑类型枚举(对齐 Rust)
    meta/
      ColumnMeta.java   # 列元数据
      IndexMeta.java    # 索引元数据
    h2/
      H2Dialect.java    # H2 实现
      H2TypeMapping.java # H2 JDBC 类型 -> aqua DataType
  src/test/java/io/aqua/connector/h2/
    H2DialectTest.java  # H2 内存库测试
```

## 3. Dialect 接口
```java
public interface Dialect {
    String name();
    Connection connect(DbConfig config) throws SQLException;
    List<String> listTables(Connection conn, String schema) throws SQLException;
    List<ColumnMeta> getColumns(Connection conn, String table) throws SQLException;
    List<IndexMeta> getIndexes(Connection conn, String table) throws SQLException;
}
```

## 4. H2 类型反解(JDBC type -> aqua DataType)
- VARCHAR/CHAR -> VARCHAR
- CLOB -> CLOB
- TINYINT -> TINYINT
- INTEGER -> INT
- BIGINT -> LONG
- DECIMAL/NUMERIC -> DECIMAL
- DATE -> DATE
- TIMESTAMP -> DATETIME
- BLOB/BINARY -> BLOB

## 5. Main 流程
1. 读 stdin 全部字节(JSON)
2. 解析请求(action/dialect/config)
3. DialectRegistry 获取 Dialect
4. connect -> 执行 action -> 构造响应 JSON
5. 写 stdout,exit 0
6. 任何异常:写 {"error":msg},exit 1

## 6. 依赖
- com.h2database:h2(测试 + 运行时,打包进 fat jar)
- com.fasterxml.jackson.core:jackson-databind(JSON)
- org.junit.jupiter:junit-jupiter(测试)

## 7. 打包
maven-shade-plugin 生成 fat jar(connector.jar,含 H2 + Jackson)。

## 8. H2 测试方案
H2 内存库(`jdbc:h2:mem:test`),Java 单元测试同进程访问:
- 建表(DDL from Rust fixture)
- 调 H2Dialect 各方法
- 断言反解结果
