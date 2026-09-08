# Design — Java 枚举类生成 + DataModel Options 映射

## 决策摘要(已与用户确认)
1. **Java 产物形态 = B(独立文件)**:每个枚举类生成独立 `.java` 文件,与实体文件并列。
2. **枚举类名默认从 prop 派生**(`gender`→`Gender`),可在枚举配置显式覆盖(`className`)。
3. **共享作用域 = C(全项目去重)**:枚举定义只生成一次,其余为引用。
4. **引用建模 = P(仅引用,无副本)**:引用方字段 `enum` 只存 `ref`(指向定义方表+字段),无 values;引用型**不生成枚举代码**,只 import/引用类型。
5. **DataModel(frontend_json)映射**:枚举字段输出 `bizType:"Options"` + `bizTypeData:[{id,name},…]`,引用方解析到定义方取值。

## 一、数据模型扩展
### `schema/enum_def.rs`
```rust
pub struct InlineEnum {
    pub name: String,
    pub has_code: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,        // 新增:显式类名,缺省按定义字段 prop 派生
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<InlineEnumRef>,      // 新增:引用型,Some ⇒ 引用方(values 可空)
    #[serde(default)]
    pub values: Vec<EnumValue>,            // 引用方为空;定义方非空
}

pub struct InlineEnumRef {
    pub code: String,   // 定义方表 code
    pub prop: String,   // 定义方字段 prop(锚点:一字段一枚举,prop 唯一稳定)
}
```
- **定义方判定**:`enum.ref == None`;**引用方**:`enum.ref == Some(_)`。
- 同表多字段共享:一个定义方 + 其余字段引用方(显式 ref),避免同表重复定义。
- 兼容:新字段全 `skip_serializing_if`/`default`,旧 schema 反序列化不变。

### 前端 `app/src/types/schema.ts`
`InlineEnum` 加 `className?: string` 与 `ref?: { code: string; prop: string }`;`values` 引用方可空。

## 二、校验(`schema/validate.rs`)
- 保留:`enum` 仅 VARCHAR;`hasCode=true` 每 value 必须有 code。
- 新增(两遍,先收集定义索引再校验引用):
  1. 收集全项目定义方索引 `(table.code, field.prop) → &Field`(仅 enum.ref=None 且有 enum)。
  2. **引用存在性**:每个引用方 `ref` 的目标 `(code, prop)` 必须存在,且目标确为定义方(有 enum 且 ref=None);目标缺失 / 指向非枚举字段 / 指向另一个引用方 ⇒ 报错。
  3. **类型一致性**:引用方字段的 `dataType` 必须与 ref 目标定义字段一致(均 VARCHAR);`length` 不一致时报错(同一枚举挂在不同宽度列上不合法)。
- 引用方不强制 values 非空(无副本)。

## 三、Java 生成器(`generators/java/`)
### 返回结构(签名变更)
```rust
pub struct JavaFile { pub path: String, pub content: String }  // path 如 "Gender.java"
pub fn generate_java_entity(project: &Project, table: &str, options: &JavaOptions)
    -> Result<Vec<JavaFile>, String>;
```
- 首项 = 实体文件;其余 = **本表定义方**枚举文件。引用型不在本表产出(定义方表负责)。
- 调用方适配:
  - `src-tauri/src/commands/generate.rs:42` `generate_java_command` 返回 `Vec<JavaFile>`(前端多文件)。
  - `crates/aqua-cli/src/commands/gen.rs:14` `entity()` 循环打印多文件(带分隔/文件名头)。

### 类型与类名
- `java_type_for(field)`:字段有 `enum` ⇒ 类型 = 枚举类名(定义方=自身派生名/`className`;引用方=解析目标定义方的类名)。替代现有对枚举字段返回 `String`。
- 类名派生 = `class_name` 或由 `prop` 首字母大写(`gender`→`Gender`);复用/扩展 `naming.rs`。
- 枚举类与实体同 package(design.md §3.5),同包无需 import。

### 枚举类输出形态(用户给定,authoritative)
普通(hasCode=false/无):
```java
enum Gender {
    MALE,  // 男
    FEMALE // 女
}
```
CodeEnum(hasCode=true):
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
- 枚举项标识 = `value.id`;`// 描述` = `value.name`;构造参数 = `value.code`。
- 末项 `;`,其余 `,`。
- **CodeEnum 接口来源**:实现前确认规范接口全限定名(疑似 `io.github.jinghui70.rainbow...`,与 entity 注解同源 `rainbow.dbaccess`),生成用 import 或全限定名,避免裸名不可编译。→ 见开放确认点。

### 数据流(单表生成)
1. 建全项目定义索引 `(code,prop) → FormattedEnum{class_name, has_code, values}`。
2. 遍历目标表字段:无 enum→原逻辑;ref=Some→查索引拿类名,字段类型=类名,不产文件;ref=None→字段类型=类名,产枚举文件。
3. 组 `Vec<JavaFile>`(实体首 + 本表定义枚举)。

## 四、DataModel 生成器(`generators/frontend_json.rs`)
- `transform_field` 增加枚举分支(需访问定义索引,故签名从 `(&Field)` → `(&Field, &EnumIndex)`,`transform_table`/入口透传 project 索引)。
- 枚举字段(定义或引用)输出:
  ```json
  "bizType": "Options",
  "bizTypeData": [ { "id": "MALE", "name": "男" }, { "id": "FEMALE", "name": "女" } ]
  ```
  - `bizType` 固定字符串 `"Options"`;`bizTypeData` = 定义方 values 的 `[{id,name}]`(仅 id+name,不含 code/color)。
  - 引用方:按 `ref` 解析到定义方取 values。
  - 非枚举字段:维持原 `biz_type`/`biz_type_data` 透传,无变化。

## 五、前端界面(`app/src/views/table-editor/BizTypeEditDialog.vue`)
枚举编辑现于 `bizType="Enum"` 展开区。新增「枚举来源」切换 + `className` + 二级级联引用选择器。

### 布局
```
业务类型  [Enum(枚举) ▾]
枚举来源  ( ) 新建定义   ( ) 引用已有            ← 新增 el-radio-group

── 模式A:新建定义 ────────────────
枚举名 [性别]   类名 [Gender]  ☑ hasCode         ← 类名新增,placeholder=prop 派生
枚举值 [+ 添加值]  (现有 id/名称/code/颜色/操作 表格,不变)

── 模式B:引用已有 ────────────────
引用表   [ USER(用户) ▾ ]                         ← 二级级联①:含定义枚举的表
引用字段 [ 性别 → Gender ▾ ]                      ← 二级级联②:该表定义方枚举字段
预览(只读) hasCode=true:MALE 男(M) / FEMALE 女(F)  ← 解析目标只读展示
```

### 交互
- **来源切换**:radio「新建定义 / 引用已有」。新建 ⇒ `enum.ref=undefined`,显示定义表单;引用 ⇒ 显示级联选择器,选定后写 `enum.ref`,清空本地 `name/hasCode/values/className`。
- **级联①「引用表」**:选项 = 全项目中含 ≥1 个定义方枚举字段(`field.enum && !field.enum.ref`)的表,label `表name(表code)`,value 表code。
- **级联②「引用字段」**:依赖①,选项 = 该表定义方枚举字段,label `字段name → 类名`,value `prop`;①未选时禁用。二者选定 ⇒ `enum.ref = { code: 表code, prop }`。排除字段自身。
- **预览**:按 `ref` 解析目标定义方,只读展示 hasCode + values。
- **长度同步**:选定引用后自动把本字段 `length` 同步为目标字段 `length`(满足后端 R6b 一致校验)。
- **className**:仅定义模式可编辑,placeholder = prop 派生 PascalCase;留空则生成时用默认。写入 `enum.className`。

### 保存校验(前端兜底,后端为准)
- 引用模式:必须已选到字段(`ref` 完整);未选报错。
- 定义模式:沿用现有(enum 仅 VARCHAR、hasCode 时 code 必填)。

### JavaTab 多文件(`JavaTab.vue` + `composables/useTauri`)
- `generateJava` 返回 `JavaFile[]`;预览区改为文件切换(下拉/标签选文件),保存支持逐个或批量导出目录;`useTauri.generateJava` 返回类型同步。

## 六、兼容性 / 迁移
- **行为变更**:枚举字段的实体类型由 `String` 变为枚举类;DataModel 枚举字段新增 `Options`。均为本特性预期变更,PRD/changelog 声明。
- 非枚举字段:Java 与 DataModel 输出完全不变(无回归)。
- 旧 schema 无 ref/className,升级即用,仅"有 enum 的字段"输出改变。

## 七、测试策略
- 复用 `tests/fixtures/valid-full.json`(已含 GENDER hasCode=true / STATUS hasCode=false);新增跨表引用 fixture(表B 字段 ref 表A GENDER)。
- `tests/` 集成测试(仿 `generators_ddl.rs` 的 `load_fixture`):
  - Java:hasCode 两分支形态、字段类型=枚举名、引用方不产文件且类型指向定义类、非枚举无回归。
  - DataModel:枚举字段 `Options`+`bizTypeData`,引用方解析,非枚举无回归。
  - 校验:ref 目标缺失 / 指向非枚举 / 指向引用方 / 类型不一致 均报错。

## 八、权衡
- 显式 `ref` vs 运行时推断:选显式(决策 P),防同名不同值误合并与撞名。
- 全项目作用域:一份定义全局唯一;将来若需按组隔离可退化(参 strconst 的 group 过滤)。
- 返回结构 `Vec<JavaFile>` 是必要破坏性变更(决策 B 独立文件的直接后果),影响 2 处调用方 + 前端 JavaTab。

## 开放确认点(实现前必须敲定,不阻塞规划结构)
- **CodeEnum 接口全限定名**:实现首步用 `rg CodeEnum`/查 rainbow-dbaccess 依赖坐标确认;确认前生成暂用占位全限定名并在 PRD 标注。
