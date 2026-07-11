# Directory Structure

> How backend code is organized in this project.

---

## Overview

aqua-core 是纯逻辑核心库,按功能模块组织。每个模块独立目录,内部按类型职责拆分文件。

---

## Directory Layout

```
crates/aqua-core/src/
├── lib.rs              # 模块声明 + 顶层文档
├── schema/             # 数据模型(Project/Table/Field/...)
│   ├── mod.rs          # pub use re-export
│   ├── data_type.rs    # DataType enum
│   ├── enum_def.rs     # EnumColor/EnumValue/InlineEnum/EnumDefine
│   ├── biz_type.rs     # BizTypeDefine/SupportedDataType/...
│   ├── field.rs        # Field/AutoGenerate/FieldEnum
│   ├── table.rs        # Table/Index
│   ├── project.rs      # Project/GroupDefine
│   └── validate.rs     # ValidationError/ParseError/validate_project/parse_project
├── generators/         # DDL/Java/JSON 生成(待移植)
├── dataset/            # 数据集载体(待移植)
├── import/             # 连库导入(待移植)
├── driver/             # Driver trait + 实现(待移植)
└── diff/               # JSON diff + ALTER 生成(待移植)

crates/aqua-core/tests/
├── schema.rs           # schema 模块集成测试
└── fixtures/           # 测试 JSON 数据
    ├── valid-full.json
    ├── invalid-enum-non-varchar.json
    ├── invalid-has-code-missing-code.json
    └── invalid-missing-required.json
```

---

## Module Organization

### 模块内拆分原则(以 schema 为例)

1. **类型文件逐个拆分**: 一个逻辑类型一个文件(data_type.rs / enum_def.rs / biz_type.rs / field.rs / table.rs / project.rs),便于对照 legacy 移植与维护
2. **校验独立文件**: validate.rs 放业务校验逻辑(ValidationError / validate_project / parse_project),与类型定义分离
3. **mod.rs 仅 re-export**: 不放业务逻辑,只 `pub use` 导出公共类型
4. **依赖顺序**: 按依赖链排列文件移植顺序(data_type → enum_def → biz_type → field → table → project → validate)

### 测试组织

- **集成测试**: `tests/<module>.rs`,测试模块对外 API
- **fixtures**: `tests/fixtures/`,JSON/二进制测试数据,移植自 legacy 或手写
- **单元测试**: 复杂函数内联 `#[cfg(test)] mod tests`,简单类型不写(serde 已覆盖)

---

## Naming Conventions

- **文件名**: 蛇形命名(snake_case),避开 Rust 关键字(enum → enum_def)
- **模块名**: 与文件名一致,mod.rs 所在目录名即模块名
- **类型名**: 大驼峰(PascalCase),对齐 legacy TS 类型名(DataType / EnumDefine / Project)
- **字段名**: 蛇形,serde `#[serde(rename)]` 映射 JSON 驼峰(base_package ↔ basePackage)
- **函数名**: 蛇形,对齐 legacy 函数名(parse_project ↔ parseProject / validate_project ↔ validateProject)

---

## Examples

**已完成**: `src/schema/` - 完整的模块拆分、类型定义、校验分离、测试覆盖范式,后续模块参照此结构。
