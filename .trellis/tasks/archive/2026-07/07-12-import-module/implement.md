# import-module 实现计划

## 实施步骤

1. [ ] `import/mod.rs` - 模块入口
2. [ ] `import/from_db.rs` - import_from_db 核心逻辑
3. [ ] 元数据转换: ColumnMeta → Field
4. [ ] 元数据转换: IndexMeta → Index
5. [ ] 默认值处理
6. [ ] 单元测试
7. [ ] 更新 lib.rs

## import_from_db 设计

```rust
pub async fn import_from_db(
    driver: &dyn Driver,
    schema: &str,
    base_package: Option<String>,
) -> Result<Project, DriverError> {
    // 1. 获取所有表
    let table_names = driver.list_tables(schema).await?;

    // 2. 逐表反解
    let mut tables = Vec::new();
    for table_name in table_names {
        let table = import_table(driver, schema, &table_name).await?;
        tables.push(table);
    }

    // 3. 构造 Project
    Ok(Project {
        base_package: base_package.unwrap_or_else(|| "com.example".to_string()),
        tables,
        enums: vec![],
        biz_types: vec![],
    })
}
```

## 元数据转换

### ColumnMeta → Field

```rust
fn column_to_field(col: ColumnMeta) -> Field {
    Field {
        code: col.name.to_uppercase(),
        name: col.comment.unwrap_or_else(|| col.name.clone()),
        prop: snake_to_camel(&col.name),
        data_type: col.data_type,
        length: col.length,
        precision: col.precision,
        scale: col.scale,
        not_null: Some(!col.nullable),
        is_key: Some(col.is_key),
        default_value: col.default_value,
        enum_ref: None,  // 暂不支持
        biz_type: None,
        biz_type_data: None,
        comment: None,
    }
}
```

### IndexMeta → Index

```rust
fn index_meta_to_index(idx: IndexMeta) -> Index {
    Index {
        name: Some(idx.name),
        fields: idx.fields.iter().map(|f| f.to_uppercase()).collect(),
        unique: idx.unique,
    }
}
```

## 默认值

- `base_package`: "com.example" (可编辑)
- `group`: "default" (可编辑)
- `Field.prop`: snake_to_camel(column_name)
- `Field.name`: comment 或 column_name

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_to_field() {
        let col = ColumnMeta {
            name: "user_name".to_string(),
            data_type: DataType::Varchar,
            length: Some(64),
            precision: None,
            scale: None,
            nullable: false,
            is_key: false,
            default_value: None,
            comment: Some("用户名".to_string()),
        };

        let field = column_to_field(col);
        assert_eq!(field.code, "USER_NAME");
        assert_eq!(field.prop, "userName");
        assert_eq!(field.name, "用户名");
    }
}
```
