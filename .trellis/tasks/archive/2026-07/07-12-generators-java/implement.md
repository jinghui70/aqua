# generators-java 实现计划

## 当前状态
- 任务状态: in_progress
- PRD: 已完成
- 实现: 待开始

## 实施步骤

1. [ ] `generators/java/mod.rs` - 模块入口
2. [ ] `generators/java/types.rs` - 类型映射(9 逻辑类型 → Java 类型)
3. [ ] `generators/java/naming.rs` - 命名转换
   - snake_case → camelCase (snakeToCamel)
   - snake_case → PascalCase (snakeToPascal)
4. [ ] `generators/java/entity.rs` - 实体类生成
   - package 声明
   - import 收集
   - 类定义 + 字段
   - getter/setter 或 @Data
5. [ ] `generators/java/annotations.rs` - 注解生成
   - @Table(name)
   - @Id (isKey=true)
   - @Column(name) (非标准命名)
   - @GeneratedValue (autoGenerate)
6. [ ] `generators/java/enum_class.rs` - 枚举类生成
   - 全局枚举
   - 内联枚举
7. [ ] `tests/generators_java.rs` - 测试

## Legacy 参考
- 类型映射: `~/work/aqua-legacy/packages/core/src/generators/java-entity/typeMap.ts`
- 命名转换: `~/work/aqua-legacy/packages/core/src/generators/java-entity/naming.ts`
- 实体生成: `~/work/aqua-legacy/packages/core/src/generators/java-entity/entityClass.ts`
- 注解生成: `~/work/aqua-legacy/packages/core/src/generators/java-entity/annotations.ts`
- 枚举生成: `~/work/aqua-legacy/packages/core/src/generators/java-entity/enumClass.ts`

## 下一步
新会话运行 `/trellis:continue` 从 Phase 2.1 开始实现。
