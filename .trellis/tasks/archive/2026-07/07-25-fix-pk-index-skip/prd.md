# 修复 JDBC 主键索引未跳过(H2 等 PK_NAME≠INDEX_NAME)

## Goal

`AbstractJdbcDialect.getIndexes` 应跳过主键索引,但在 H2 等库上跳过失效,导致导入 schema 多出一条覆盖主键列的伪唯一索引。

## 背景(发现于 07-25-import-batch-feedback 任务的 importTables 单测)

- `getIndexes`(`connector/.../AbstractJdbcDialect.java:145`)用 `getPrimaryKeys().PK_NAME` 匹配 `getIndexInfo().INDEX_NAME` 来跳过主键索引。
- H2 上二者不相等:`PK_NAME` 是约束名(或 null),`INDEX_NAME` 形如 `PRIMARY_KEY_A` → 匹配失败 → 主键索引未跳过。
- 实测:`SYS_ORDER(ID PK, AMOUNT)` 反解出 `indexes:[{name:"PRIMARY_KEY_F", fields:["ID"], unique:true}]`——纯属多余(主键已由 `ColumnMeta.isKey` 表达)。
- **预先存在**,与批量化无关:旧单表 `list_indexes` 路径同样中招;既有 `H2DialectTest.testGetIndexes` 只 filter 断言目标索引存在,未断言主键索引缺席,故漏网。

## 影响范围
- 受影响:PK 约束名 ≠ PK 索引名的 JDBC 库(H2 已确认;信创库需实测)。
- Oracle 通常 PK 约束名 == 索引名,可能不受影响(待确认)。
- native MySQL/PG 不走此逻辑(自有 `PRIMARY` 跳过),不受影响。

## Requirements
- `getIndexes` 可靠跳过主键索引,不依赖 PK_NAME==INDEX_NAME 的脆弱假设。
  - 候选:比对主键列集合与索引列集合(主键索引 = 列集与 PK 列集相同的唯一索引),或用 `getIndexInfo` 结合 `getPrimaryKeys` 的列而非名。
- 不误伤业务上真实存在、恰好覆盖主键列的额外唯一索引(需甄别:同列集且为 PK 自动索引才跳)。

## Acceptance Criteria
- [ ] H2:`SYS_USER`/`SYS_ORDER` 反解 indexes 不含 `PRIMARY_KEY_*`。
- [ ] 保留真实非主键索引(如 `IDX_SYS_USER_USER_NAME`)。
- [ ] `H2DialectTest` 补主键索引缺席断言;`ImportTablesTest` 相应收紧。
- [ ] Oracle/信创路径行为不回归(至少 H2 + 通用逻辑验证)。

## Notes
- 优先级:中(污染导入结果但不阻断;用户可手删多余索引)。
- 修复应在 `AbstractJdbcDialect` 统一处理,惠及所有 JDBC 方言。
