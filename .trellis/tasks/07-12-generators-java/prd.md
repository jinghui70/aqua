# generators 模块: Java 实体生成器

## 背景

aqua v2 移植路线 P0 第二步。Java 实体生成器验证业务类型系统(bizType)和枚举(Enum)的正确性。

- 逻辑蓝本: `~/work/aqua-legacy/packages/core/src/generators/java-entity/`
- 业务规范: `docs/design.md` §4.2.1 Java 实体生成规则
- 现状: DDL 生成器已完成,Java 生成器待实现

## 目标

实现 Java 实体生成器,按 rainbow-dbaccess 规范生成实体类 + 枚举类。

**包含**:
- 实体类生成: package/类名/字段/getter/setter
- 类型映射: 9 种逻辑类型 → Java 类型
- 注解生成: @Table/@Id/@Column/@GeneratedValue
- 枚举类生成: 全局枚举 + 内联枚举
- Lombok 支持: @Data 注解(可选)
- 命名转换: 表名/字段名 → PascalCase/camelCase

## 范围(不含)

- Javadoc 生成(comment → 注释,本期简化不做)
- 多表批量生成(CLI 循环调用单表生成)
- 文件写入(返回 String,CLI 负责写文件)

## 验收标准

**实现**:
- [ ] `generators/java/mod.rs` 模块入口
- [ ] `generators/java/types.rs` 类型映射(9 逻辑类型 → Java 类型)
- [ ] `generators/java/naming.rs` 命名转换(snake_case → PascalCase/camelCase)
- [ ] `generators/java/entity.rs` 实体类生成
- [ ] `generators/java/enum_class.rs` 枚举类生成
- [ ] `generators/java/annotations.rs` 注解生成(@Table/@Id/@Column)

**测试**:
- [ ] `tests/generators_java.rs` 集成测试
- [ ] valid-full.json → Java 实体往返(生成 → 编译验证)
- [ ] 枚举类生成验证(全局/内联)

**质量**:
- [ ] `cargo test -p aqua-core` 全绿
- [ ] `cargo clippy -p aqua-core -- -D warnings` 无 warning
- [ ] 生成的 Java 代码可编译(手动验证 javac)

## 约束

- 纯逻辑,无 I/O(返回 String)
- 类型映射遵循 design.md §4.2.1 规则
- 注解遵循 rainbow-dbaccess 规范
- package 规则: `{basePackage}.{groupCode}.{entityName}`

## 参考

- legacy 实现: `~/work/aqua-legacy/packages/core/src/generators/java-entity/`
- 类型映射: design.md §4.2.1
- 测试用例: legacy `__tests__/generators/java-entity.test.ts`
