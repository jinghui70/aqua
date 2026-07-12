# 枚举类 Java 生成 (全局枚举)

## 背景
generators-java 当初 PRD 写了"枚举类生成"但实现遗漏(entity.rs 只生成实体)。用户指出枚举管理页应显示对应 Java 代码。

## 目标
后端补全局枚举 Java 类生成 + Tauri command,前端枚举管理页加 Java 预览。

## 包含
### 后端(aqua-core)
- generators/java/enum_class.rs: generate_global_enum_class(project, enum_def)
- hasCode=true: `MALE("M","男")` + implements CodeEnum + code/name 字段 + getter
- hasCode=false: 普通枚举 `MALE // 男`
- package = {basePackage}.{enumDef.package}
- 单元测试(两种 hasCode)

### Tauri command
- generate_enum_command(project, enumCode) -> String

### 前端
- EnumManage 右侧加 "Java 预览" 区(选中枚举实时预览 + 复制)

## 范围(不含)
- 内联枚举类生成(表编辑器字段引用场景,后续)
- 批量导出所有枚举(导出菜单,fe-export)

## 验收标准
- [ ] enum_class.rs + generate_global_enum_class
- [ ] 单元测试: hasCode true/false 两种
- [ ] generate_enum_command 注册
- [ ] EnumManage Java 预览区(实时 + 复制)
- [ ] cargo test + pnpm build 通过

## 参考
- legacy: ~/work/aqua-legacy/packages/core/src/generators/java-entity/enumClass.ts
- 规则: design.md §3.5
