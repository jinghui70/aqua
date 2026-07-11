# schema 模块技术设计

## 1. 核心决策: 类型与校验分离

### 问题
legacy 用 zod 实现"类型定义 + 运行时校验同源":一个 zod schema 既派生 TS 类型(`z.infer`),又做运行时校验(`superRefine`)。

### Rust 方案: 两层分离
Rust 无此机制。采用职责分离:
- **类型层(serde derive)**: 结构体 + serde 反序列化,管"结构正确"--字段名/类型/必填可选/枚举变体。serde 自带的 unknown 字段忽略、类型不匹配报错、可选字段默认 None 已覆盖 legacy 大量 `min(1)`/optional 规则的结构部分。
- **业务校验层(独立 `validate_project` 函数)**: 管"业务规则"--enum 只支持 VARCHAR、hasCode 一致性、values 非空等跨字段/语义规则。

比 legacy 的 superRefine 职责更清晰,是 Rust 惯例。serde 做不了的语义校验放 validate。

> 注: legacy 的 `min(1)` 字符串非空校验,在 Rust 里 serde 不直接做(serde 只校验字段存在与否)。空字符串 `""` 能反序列化成功。因此"必填非空"规则也归入 validate 层统一处理,而非散落在 serde。

## 2. 对外 API
```rust
// 反序列化(JSON Value -> Project),纯结构层
Project::from_json(value: serde_json::Value) -> Result<Project, serde_json::Error>

// 业务校验,收集所有错误(不短路,对齐 legacy 一次返回全部 errors)
validate_project(project: &Project) -> Result<(), Vec<ValidationError>>

// 反序列化 + 校验合一(常用入口,对齐 legacy parseProject)
parse_project(value: serde_json::Value) -> Result<Project, ParseError>
```

## 3. 关键类型映射

### DataType (9 变体,大写)
```rust
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataType { Varchar, Clob, Tinyint, Int, Long, Decimal, Date, Datetime, Blob }
```

### Field.enum 联合类型 (string | object)
```rust
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum FieldEnum {
    Ref(String),           // 引用全局枚举 code
    Inline(InlineEnum),    // 内联枚举
}
```
`#[serde(untagged)]` 按变体顺序尝试: 先 String,失败再 InlineEnum。

### Field.bizTypeData (legacy z.unknown())
```rust
pub bizTypeData: Option<serde_json::Value>,  // 单 field 存值、多 field 存对象,校验宽松
```

### AutoGenerate.timing 枚举
```rust
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenerateTiming { Insert, InsertUpdate }
```

## 4. 错误类型 (thiserror)
```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("JSON 反序列化失败: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("Project 校验失败: {0} 个错误")]
    Validate(Vec<ValidationError>),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ValidationError {
    pub path: String,      // 如 "tables[0].fields[1].enum"
    pub message: String,
}
```
ValidationError 带 path + message,对齐 legacy errors 结构,前端可定位字段。

## 5. 模块组织
```
crates/aqua-core/src/schema/
  mod.rs          // pub use re-export
  data_type.rs    // DataType
  field.rs        // AutoGenerate, FieldEnum, Field
  table.rs        // Index, Table
  biz_type.rs     // SupportedDataType, BizTypeDataField, BizTypeData, BizTypeDefine
  enum_def.rs     // EnumColor, EnumValue, InlineEnum, EnumDefine  (enum 是 Rust 关键字,文件名避让)
  project.rs      // GroupDefine, Project
  validate.rs     // ValidationError, ParseError, validate_project, parse_project, Project::from_json
```
文件名逐一对齐 legacy `schema/*.ts`,便于对照移植。

## 6. 测试
- 移植 legacy `__tests__/schema.test.ts` 6 个用例到 `crates/aqua-core/tests/schema.rs`
- fixtures 移植到 `crates/aqua-core/tests/fixtures/`: valid-full / invalid-enum-non-varchar / invalid-has-code-missing-code / invalid-missing-required
- fixture 读取: 测试 helper + `std::fs::read_to_string`(集成测试可访问文件系统)
- 新增 serde 往返测试: valid-full JSON -> Project -> JSON,与原 JSON 比较(字段顺序由 serde 保证)

## 7. 与 legacy 的有意差异
| 点 | legacy (zod) | Rust | 原因 |
|---|---|---|---|
| 类型+校验 | 同源 superRefine | serde + validate 分离 | Rust 无 zod 机制,职责更清晰 |
| 字符串非空 | `z.string().min(1)` | validate 层统一判 `is_empty()` | serde 不做非空,集中校验 |
| 错误返回 | `{success,data,errors}` | `Result<>` + `Vec<ValidationError>` | Rust Result 惯例 |
| bizTypeData | `z.unknown()` | `serde_json::Value` | 等价,保留任意 JSON |
