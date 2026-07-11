# generators-ddl 技术设计

## 1. 模块结构

```
crates/aqua-core/src/generators/
├── mod.rs                  # pub use ddl
└── ddl/
    ├── mod.rs              # 入口: generate_ddl() + Dialect enum
    ├── types.rs            # 类型映射表: dialectMap
    ├── table.rs            # CREATE TABLE 生成
    ├── index.rs            # CREATE INDEX 生成
    └── insert.rs           # INSERT 生成(数据集支持)
```

## 2. 核心类型

### Dialect enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Mysql,
    Postgresql,
    Oracle,
    Dm,       // 达梦
    Kingbase, // 人大金仓
    Gbase,    // 南大通用
    H2,
}
```

### 类型映射

```rust
// types.rs
pub fn map_type(data_type: DataType, field: &Field, dialect: Dialect) -> String {
    match (dialect, data_type) {
        (Dialect::Mysql, DataType::Varchar) => format!("VARCHAR({})", field.length.unwrap_or(255)),
        (Dialect::Mysql, DataType::Long) => "BIGINT".to_string(),
        // ... 9 类型 × 7 方言 = 63 分支
    }
}
```

## 3. CREATE TABLE 生成逻辑

### 字段定义

```rust
fn field_definition(field: &Field, dialect: Dialect) -> String {
    let name = field.code.to_uppercase();
    let data_type = map_type(field.data_type, field, dialect);
    let not_null = if field.not_null.unwrap_or(false) { " NOT NULL" } else { "" };
    let default = field.default_value.as_ref()
        .map(|v| format!(" DEFAULT {}", v))
        .unwrap_or_default();
    let comment = if inline_column_comment(dialect) {
        format!(" COMMENT '{}'", escape_sql_string(&field.name))
    } else {
        String::new()
    };
    
    format!("{} {}{}{}{}", name, data_type, not_null, default, comment)
}
```

### PRIMARY KEY

```rust
fn primary_key_clause(table: &Table) -> Option<String> {
    let keys: Vec<_> = table.fields.iter()
        .filter(|f| f.is_key.unwrap_or(false))
        .map(|f| f.code.to_uppercase())
        .collect();
    
    if keys.is_empty() {
        None
    } else {
        Some(format!("  PRIMARY KEY ({})", keys.join(", ")))
    }
}
```

### COMMENT 方言分支

- **内联列注释** (MySQL/H2/DM/GBase): `COMMENT 'name'` 在字段定义内
- **独立列注释** (PG/Oracle/KingBase): `COMMENT ON COLUMN table.field IS 'name'`
- **内联表注释** (MySQL): `CREATE TABLE (...) COMMENT 'name'`
- **独立表注释** (其他): `COMMENT ON TABLE table IS 'name'`

## 4. CREATE INDEX 生成

```rust
pub fn generate_index(table: &Table, index: &Index) -> String {
    let name = index.name.as_ref()
        .map(|n| n.to_uppercase())
        .unwrap_or_else(|| auto_index_name(&table.code, &index.fields));
    
    let unique = if index.unique { "UNIQUE " } else { "" };
    let fields = index.fields.iter()
        .map(|f| f.to_uppercase())
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("CREATE {}INDEX {} ON {} ({});",
        unique, name, table.code.to_uppercase(), fields)
}

fn auto_index_name(table: &str, fields: &[String]) -> String {
    format!("IDX_{}_{}", table, fields.join("_")).to_uppercase()
}
```

## 5. 主入口

```rust
pub fn generate_ddl(project: &Project, options: &DdlOptions) -> String {
    let dialect = options.dialect.unwrap_or(Dialect::Mysql);
    let tables = filter_tables(project, options);
    
    let mut output = Vec::new();
    
    for table in tables {
        // CREATE TABLE + COMMENT
        output.push(generate_table(table, dialect));
        
        // CREATE INDEX
        if let Some(indexes) = &table.indexes {
            for index in indexes {
                output.push(generate_index(table, index));
            }
        }
    }
    
    output.join("\n\n") + "\n"
}
```

## 6. 选项与过滤

```rust
pub struct DdlOptions {
    pub dialect: Option<Dialect>,
    pub tables: Option<Vec<String>>,  // 表名过滤
    pub group: Option<String>,        // 分组过滤(与 tables 互斥)
}

fn filter_tables<'a>(project: &'a Project, options: &DdlOptions) -> Vec<&'a Table> {
    if let Some(ref table_codes) = options.tables {
        let set: HashSet<_> = table_codes.iter().collect();
        project.tables.iter()
            .filter(|t| set.contains(&t.code.as_str()))
            .collect()
    } else if let Some(ref group) = options.group {
        project.tables.iter()
            .filter(|t| t.group == *group)
            .collect()
    } else {
        project.tables.iter().collect()
    }
}
```

## 7. SQL 转义

```rust
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")  // 单引号 -> ''
}
```

## 8. 与 legacy 的差异

| 方面 | legacy (TS) | Rust |
|------|-------------|------|
| 类型映射 | Record<Dialect, Record<DataType, TypeMapper>> | match (dialect, data_type) |
| 注释方言 | Set<Dialect> 判断 | const fn inline_xxx(dialect) |
| 字符串拼接 | 模板字符串 | format! |
| 过滤表 | filter + Set | filter + HashSet |
| 错误处理 | 无(输入已校验) | 无(输入已 validate_project) |

## 9. 测试策略

### 单元测试

- `map_type`: 逐个断言 9×7=63 分支
- `escape_sql_string`: SQL 注入防护
- `auto_index_name`: 命名规则
- `field_definition`: 字段定义格式

### 集成测试

```rust
#[test]
fn test_mysql_ddl() {
    let project = load_fixture("valid-full.json");
    let ddl = generate_ddl(&project, &DdlOptions {
        dialect: Some(Dialect::Mysql),
        ..Default::default()
    });
    
    assert!(ddl.contains("CREATE TABLE SYS_USER"));
    assert!(ddl.contains("BIGINT"));  // LONG -> BIGINT
    assert!(ddl.contains("COMMENT '用户'"));
}
```

## 10. 性能考虑

- 纯 CPU,无 I/O,性能不是瓶颈
- 字符串拼接用 `Vec<String>` + `join()`,避免多次 realloc
- 类型映射用 match,编译期展开,零运行时开销

## 11. 未来扩展

- INSERT 生成(数据集支持) - 留 insert.rs 占位,本任务不实现
- 自定义模板 - 暂不支持,硬编码格式
- SQL 格式化选项(缩进/换行) - 固定格式
