# Implement: AquaH2 测试工具类

## 实现清单

1. **新建 `connector/src/test/java/com/aqua/aquah2/AquaH2.java`**(单文件):
   - record POJO: `Project`/`Table`/`Field`/`Index`/`IndexField`/`Group`(对齐 aqua schema)。
   - 构造: `AquaH2(String aquaJson)` / `AquaH2(InputStream)` -> Jackson 解析 Project。
   - fluent: `dataset(String/InputStream)` 解析 JSONL;`table(String...)`/`group(String...)` 累加。
   - `export()`: 过滤集解析 -> 逐表 CREATE TABLE + CREATE INDEX + COMMENT -> 数据集 INSERT。
   - `mapH2(Field)`: 类型映射(照搬 aqua-core `map_h2`)。
   - `literal(Object)`: 值字面量转义(字符串 `''` / NULL / 数字直 / 时间字符串)。
2. **新建 `connector/src/test/java/com/aqua/aquah2/AquaH2Test.java`**:
   - 读 `frs.aqua` + `frs.test.data`(放 `src/test/resources/aquah2/`)。
   - export 全部表 -> H2 内存库执行 -> 断言建表 + 插数据成功。
   - table/group 过滤测试(只建/插指定表)。
3. **测试数据**: 造脱敏 `test.aqua` + `test.data`(com.example,含 10 种类型 + default + 索引 + 两表)放 `connector/src/test/resources/aquah2/`。frs 真实 .aqua/.data 含公司信息(cn.com.yusys),**不提交**。

## 验证命令

- `cd connector && mvn test-compile` - 编译通过
- `cd connector && mvn test -Dtest=AquaH2Test` - H2 实跑建表 + 插数据验证

## 手动验证(copy frs)

- copy `AquaH2.java` 到 `~/work/frs/backend/common-test/src/main/java/.../`,改包名。
- frs 测试调 `new AquaH2(...).export()` 建 H2 内存库。

## 风险点

- record 字段对齐 .aqua schema(漏字段 Jackson 忽略,安全;test 验证)。
- H2 2.2.224 语法 -> test 实跑验证(已通过)。
- 测试数据 test.aqua/test.data 脱敏(com.example),放 connector test resources。frs 真实 .aqua/.data 含公司信息(cn.com.yusys),**不提交**。
- DEFAULT 智能处理:裸字符串自动加引号;数字/布尔/NULL/已引号/函数原值。
