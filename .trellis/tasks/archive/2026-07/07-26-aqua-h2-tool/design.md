# Design: AquaH2 测试工具类

## 架构与边界

- **位置**: `connector/src/test/java/com/aqua/aquah2/AquaH2.java`(单文件,含 record POJO)。放 test 表明是测试工具,不进 connector jar(shade 只 main);源码 copy 给 frs common-test。
- **依赖**: 仅 Jackson(`jackson-databind`,connector pom compile scope,test 可用)。不依赖 Spring/H2 JDBC/rainbow-dbaccess。H2 仅用于 `AquaH2Test` 跑内存库建表验证。
- **纯逻辑**(D2): 只接 String/InputStream(内容),出 SQL String。不碰 IO/不建库。

## 数据模型(record POJO,对齐 aqua schema)

```java
record Project(List<Table> tables, List<Group> groups)
record Table(String code, String name, String group, List<Field> fields, List<Index> indexes, String comment)
record Field(String code, String prop, String name, String dataType,
             Integer length, Integer precision, Integer scale,
             Boolean isKey, Boolean notNull, String defaultValue, String comment)
record Index(String name, List<IndexField> fields, boolean unique)
record IndexField(String code, String direction)
record Group(String code, String name)
```

Jackson 反序列化 .aqua -> Project。未知字段忽略(record 漏字段不影响)。

## 数据流

1. **构造** `new AquaH2(aquaJson)` -> Jackson 解析 Project(绑主文件,解析一次)。
2. **配置**(fluent): `dataset(jsonl)` 解析 JSONL 存 `List<DataRow>`;`table(...)`/`group(...)` 累加过滤集。
3. **`export()`**:
   - 解析过滤集(table 累加 + group 内表,合并去重;都空=全部表)。
   - 逐表: `CREATE TABLE`(列类型 `mapH2` + 长度/精度/主键/notNull/defaultValue) + `CREATE INDEX` + `COMMENT ON COLUMN/TABLE`。
   - 数据集: 按过滤表范围,逐行 `INSERT`(值字面量转义)。
   - 返回 SQL String。

## H2 类型映射(D4,照搬 aqua-core `map_h2`)

| 逻辑类型 | H2 |
|---|---|
| VARCHAR | `VARCHAR(length 或 255)` |
| CLOB | `CLOB` |
| TINYINT | `TINYINT` |
| INT | `INT` |
| LONG | `BIGINT` |
| DECIMAL | `DECIMAL(p,s)` / `DECIMAL`(无 p) |
| DOUBLE | `DOUBLE` |
| DATE | `DATE` |
| DATETIME | `TIMESTAMP` |
| BLOB | `BLOB` |

## table/group 过滤(R4)

- table 累加 + group 内的表(`group code` -> `tables where table.group == code`),合并去重(按 table code)。
- 都空 = 全部表。
- 数据集只插过滤范围内表的数据。

## INSERT 值字面量转义

- 字符串: 单引号包裹,内部 `'` 转义为 `''`。
- `null`: `NULL`。
- 数字/布尔: 直接。
- 时间(Date/Datetime 存为字符串): 字符串字面量(H2 解析)。

## 空表处理

.aqua 有但 .data 没有的表: 只生成 DDL,不生成 INSERT(数据集无该表行,自然跳过)。

## DDL 细节决策

- **DEFAULT**: 数字/布尔/NULL/已引号/函数(含括号或 CURRENT_/NOW)原值;裸字符串(用户未加引号)自动加引号,兼容 .aqua 数据。
- **INDEX**: `CREATE [UNIQUE] INDEX name ON table(col [ASC|DESC])`(name 为空则自动生成 `IDX_<table>_<col>`)。
- **DROP TABLE IF EXISTS**: **不带**(内存库每次新建,无需 DROP;prd 未要求。如需 DbaBuilder 自行处理)。

## 兼容性

- AquaH2 单文件,copy 给 frs 后改包名即可,不依赖 aqua 其他代码。
- connector jar 分发不受影响(AquaH2 在 test,不进 jar)。

## 风险

- H2 COMMENT/INDEX 语法版本差异 -> `AquaH2Test` 用 connector 的 h2 2.2.224 实跑验证。
- .data 值类型(JSON 原生): 用 `Object`/`JsonNode` 承载 row 值,转字面量时按类型分发。
- record POJO 字段对齐 .aqua schema(漏字段 Jackson 忽略,安全)。
