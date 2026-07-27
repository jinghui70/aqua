# AquaH2 测试工具类

## Goal

在 aqua 项目提供 **`AquaH2`** Java 类:读 `.aqua`(表结构)+ `.data`(数据集),生成可直接在 H2 执行的 DDL + INSERT SQL。供消费项目(如 frs)的测试基础设施 copy 源码使用,支撑"建 H2 内存表跑单元测试"。

## Background

- **决策渊源**(aqua-cli-skill task D5):测试建 H2 内存表走 Java 实现,不复用 aqua-core 的 Rust DDL 生成器。跨语言复用三条路(spawn/预生成/重写)均有代价;H2 转换稳定 + H2 是 Java 生态原生 -> Java 重写代价最低。
- **三层架构**(与用户确认):
  | 层 | 职责 | 归属 | 依赖 |
  |---|---|---|---|
  | **AquaH2**(底层) | 解析 .aqua/.data,生成 H2 SQL | **aqua**(copy 源码) | 仅 Jackson |
  | **DbaBuilder**(中间层) | IO(classpath 找文件)+ 调 AquaH2 + 建 MemoryDba | frs common-test | Spring + H2 + dba |
  | **Test**(消费层) | builder 调用 | frs 业务测试 | DbaBuilder |
- **aqua-core H2 映射已确认完整**:`generators/ddl/types.rs:map_h2` 10 种逻辑类型全覆盖,Java 照搬。
- **frs common-test 现状**:`BaseTest`/`DbaConfig` 空壳;resources 有 `frs.aqua` + `frs.test.data`;命名约定 `<aquaFile>.aqua` / `<aquaFile>.<dataset>.data`。

## Decisions

- **D1 归属与分发**:AquaH2 放 aqua 项目,随 schema 进化;**不分发 jar**,消费项目 copy 源码。单源维护 + 零分发成本。
- **D2 纯逻辑**:AquaH2 不碰 IO、不绑 Spring、不建库。只接 String/InputStream(内容),出 SQL String。建库是 DbaBuilder 的活。
- **D3 builder 接口**:构造绑主文件;dataset/table/group fluent 配置;**唯一 `export()` 出口**。table/group 都空 = 全部表;可混合(累加,export 时合并去重)。
- **D4 H2 映射照搬 `map_h2`**:Varchar->VARCHAR(n) 默认 255、Clob->CLOB、Tinyint->TINYINT、Int->INT、Long->BIGINT、Decimal->DECIMAL(p,s)、Date->DATE、Datetime->TIMESTAMP、Blob->BLOB、Double->DOUBLE。
- **D5 依赖**:仅 Jackson(JDK 17 record 定义 POJO)。不依赖 Spring/H2 JDBC/rainbow-dbaccess。

## Interface

```java
public class AquaH2 {
    // 构造(绑主文件,解析一次)
    public AquaH2(String aquaJson) { ... }
    public AquaH2(InputStream aquaIn) { ... }

    // 配置(fluent)
    public AquaH2 dataset(String jsonl) { ... }
    public AquaH2 dataset(InputStream in) { ... }
    public AquaH2 table(String... tables) { ... }   // 累加表
    public AquaH2 group(String... groups) { ... }    // 累加组

    // 唯一导出:table/group 都空 = 全部表
    public String export() { ... }
}
```

消费侧(DbaBuilder,frs)内部用法:
```java
AquaH2 h2 = new AquaH2(readClasspath(file + ".aqua"));
if (dataset != null) h2.dataset(readClasspath(file + "." + dataset + ".data"));
h2.table(tables).group(groups);
String sql = h2.export();
return createMemoryDba(sql);
```

## Requirements

- **R1 解析 .aqua**:Jackson 反序列化 JSON -> Java record 模型(Project/Table/Field/Index),JDK 17 record 单文件。
- **R2 生成 H2 DDL**:CREATE TABLE(列类型按 D4 映射 + 长度/精度/主键/notNull)+ CREATE INDEX。表/列 COMMENT(H2 语法)。
- **R3 解析 .data + 生成 INSERT**:JSONL 每行 `{table, row}`,按 export 的表范围过滤,生成 INSERT 语句(值转字面量)。
- **R4 table/group 过滤**:table 累加 + group 内的表,合并去重;都空 = 全部表。数据集只插过滤范围内表的数据。
- **R5 单文件**:AquaH2 + record POJO 一个文件,消费项目 copy 单文件即用。

## Acceptance Criteria

- [ ] AquaH2 类实现,仅依赖 Jackson,JDK 17 编译通过。
- [ ] 对 `~/work/frs/.../frs.aqua` + `frs.test.data`,export 全部表 -> 生成的 SQL 能在 H2 内存库建表 + 插数据成功。
- [ ] table/group 过滤正确(只建指定表,只插指定表数据)。
- [ ] 类型映射与 aqua-core `map_h2` 一致(D4)。
- [ ] 单文件,可独立编译(copy 后不依赖 aqua 其他代码)。

## Out of Scope（本期不做）

- **DbaBuilder**:IO + 建 MemoryDba,归 frs common-test。
- **Maven 分发**:不发 jar,消费项目 copy 源码。
- **aqua-core h2 dialect 清理**:aqua-core 的 `map_h2` 是否还有其他消费者(GUI 导出),本期不动。

## Open Questions (进 design 定)

- AquaH2 放 aqua 哪个目录?怎么编译验证不破坏(connector 已有 Maven + H2,复用还是新建模块)?
- H2 DDL 细节:COMMENT 语法(`COMMENT ON COLUMN`)、INDEX 语法、DROP TABLE IF EXISTS 前缀?
- 数据集值字面量转义(字符串引号、null、数字、时间)?
- .aqua schema 里 .data 没有的表(空表,只 DDL 不 INSERT)。
