# PRD — Java 枚举类生成 + DataModel Options 映射

## Goal
字段配置为内联枚举(`field.enum`)时:
1. Java 生成器除实体类外,生成对应的独立 Java 枚举类,并让实体字段引用该枚举类型(替代现有 `String`)。
2. 跨表可共享同一枚举:引用方只记录引用、不重复生成代码。
3. DataModel(前端 JSON)对枚举字段输出 `bizType:"Options"` 形态。

落地并修订 `docs/design.md` §3.5「内联枚举生成独立 enum 类」。

## Background / 现状(代码证据)
- 数据模型:`crates/aqua-core/src/schema/enum_def.rs` — `InlineEnum { name, has_code, values }`,`EnumValue { id, name, code?, color? }`。
- 字段挂载:`Field.enum_ref`(JSON key `enum`),`field.rs:64-66`。
- 现有校验:`validate.rs:167-195` — `enum` 仅 VARCHAR;`hasCode=true` 每 value 必须有 code。
- **缺口**:`generators/java/entity.rs` 完全忽略 `enum_ref`——`java_type_for()`(entity.rs:143)对枚举字段返回 `String`,不生成枚举类;`generators/frontend_json.rs` `transform_field` 不处理 `enum`。
- 输出形态:`generate_java_entity` 返回单 `String`(mod.rs:14);前端 `JavaTab.vue` 单文本框预览、存单文件。调用方:`src-tauri/.../generate.rs:42`、`aqua-cli/.../gen.rs:21`。
- 样例(`tests/fixtures/valid-full.json`):`GENDER`/prop=`gender`/hasCode=true/[MALE(男,M),FEMALE(女,F)];`STATUS`/prop=`status`/hasCode=false/[ACTIVE(启用)]。

## 关键决策(已确认)
- **D1 产物形态**:独立 `.java` 文件(每个枚举一个),非嵌套。`generate_java_entity` 返回 `Vec<JavaFile{path,content}>`。
- **D2 类名**:默认由定义字段 `prop` 派生 PascalCase(`gender`→`Gender`),可经 `InlineEnum.className` 显式覆盖。
- **D3 共享作用域**:全项目去重,每个枚举定义全局只生成一次。
- **D4 引用建模**:仅引用无副本。引用方字段 `enum` 存 `ref{code,prop}`(指向定义方表+字段),不含 values;引用型不生成代码,字段类型指向定义方枚举类。
- **D5 DataModel**:枚举字段输出 `bizType:"Options"` + `bizTypeData:[{id,name},…]`;引用方解析定义方取值。

## 目标 Java 输出形态(用户给定,authoritative)
普通枚举(hasCode=false/无,DB 存 id):
```java
enum Gender {
    MALE,  // 男
    FEMALE // 女
}
```
CodeEnum(hasCode=true,DB 存 code):
```java
public enum Gender implements CodeEnum {
    MALE("M"),   // 男
    FEMALE("F"); // 女

    private final String code;

    Gender(String code) { this.code = code; }

    @Override
    public String code() { return code; }
}
```
映射:枚举项标识=`EnumValue.id`;`// 描述`=`EnumValue.name`;构造参数=`EnumValue.code`。
> 该形态修订 §3.5 原文 `MALE("M","男")`(描述放进构造参数)。以本 PRD 为准。

## 目标 DataModel 形态(用户给定,authoritative)
```json
"bizType": "Options",
"bizTypeData": [
  { "id": "MALE", "name": "男" },
  { "id": "FEMALE", "name": "女" }
]
```

## Requirements
- R1:枚举字段(定义或引用)的实体字段类型 = 派生枚举类名(如 `private Gender gender;`)。
- R2:定义方枚举生成独立枚举类,形态见上。
  - R2a:hasCode=false/无 → 普通枚举,项后接 `// {name}`。
  - R2b:hasCode=true → `implements CodeEnum`,单参构造(`code`)+ `code` 字段 + `code()` 覆写,项后接 `// {name}`;末项 `;` 其余 `,`。
- R3:类名按 D2(prop 派生 + `className` 覆盖)。
- R4:枚举类与实体同 package。
- R5:数据模型扩展——`InlineEnum` 加 `className?`、`ref?{code,prop}`,`values` 引用方可空;前端 `schema.ts` 同步。
- R6:校验扩展(`validate.rs`):
  - R6a:引用目标存在,且为定义方(有 enum 且 ref=None);目标缺失/指向非枚举/指向引用方 → 报错。
  - R6b:引用方字段 `dataType`(及 `length`)必须与 ref 目标定义字段一致 → 否则报错。
- R7:DataModel 按 D5 输出 `Options`。
- R8:`CodeEnum` 接口以全限定名生成(已确认:`io.github.jinghui70.rainbow.dbaccess.object.CodeEnum`)。
- R9:同步更新 `docs/design.md` §3.5(Java 示例 + ref/className + DataModel Options 说明)。
- R10:前端枚举编辑(`BizTypeEditDialog.vue`)支持:
  - R10a:「枚举来源」切换——新建定义 / 引用已有。
  - R10b:定义模式加 `className` 输入(placeholder=prop 派生,留空用默认)。
  - R10c:引用模式二级级联——先选「引用表」(含定义枚举的表)、再选「引用字段」(该表定义方枚举字段,label `字段名 → 类名`),选定写 `enum.ref{code,prop}` 并只读预览目标;选定后同步本字段 `length`。
  - R10d:`JavaTab.vue` 适配 `JavaFile[]` 多文件预览 / 保存。

## Acceptance Criteria
- AC1:hasCode=false 枚举字段——实体字段类型=枚举类名,并生成 R2a 普通枚举类,注释与项对应。
- AC2:hasCode=true 枚举字段——生成 R2b CodeEnum 类,构造参数取 code、注释取 name,末项 `;` 其余 `,`。
- AC3:类名对 `prop=gender` 派生 `Gender`;给定 `className` 时用之。
- AC4:跨表引用——引用方实体字段类型指向定义方枚举类,引用方**不产枚举文件**,定义方产且仅产一次。
- AC5:DataModel——枚举字段输出 `bizType:"Options"` + `bizTypeData:[{id,name}]`;引用方解析定义方取值。
- AC6:校验——ref 目标缺失/指向非枚举/指向引用方/类型或长度不一致,均返回校验错误。
- AC7:非枚举字段的 Java 与 DataModel 输出与当前完全一致(无回归)。
- AC8:`cargo test -p aqua-core` 全绿;新增枚举生成/校验单测覆盖 hasCode 两分支 + 跨表引用 + 校验错误。
- AC9:前端枚举弹窗——可切换新建/引用;引用模式二级级联(表→字段)选定后写 `ref` 并预览目标;定义模式可填 `className`;JavaTab 多文件预览/保存正常。

## Out of Scope
- 全局枚举/枚举管理页(design.md 明确"无全局枚举")。
- DDL / strconst 生成器的枚举改动(本任务仅 Java 实体 + DataModel)。

## Technical Notes
- 定义索引 `(table.code, field.prop) → EnumMeta{class_name, has_code, values}` 为 Java 与 frontend_json 共用。
- 返回结构 `Vec<JavaFile>` 是破坏性变更,牵动 tauri/cli 调用方 + 前端 JavaTab,集中一提交改完再验。
- 设计细节见 `design.md`,执行清单见 `implement.md`。
