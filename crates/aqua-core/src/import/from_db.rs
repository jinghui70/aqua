//! 从数据库导入 schema 核心逻辑。

use crate::driver::{Driver, DriverError, ColumnMeta, IndexMeta};
use crate::schema::{Project, Table, Field, Index};
use crate::generators::java::naming::snake_to_camel;

/// 从数据库导入 schema,生成 Project。
///
/// # 参数
/// - `driver`: 数据库驱动
/// - `schema`: schema/database 名称
/// - `base_package`: 基础包名(默认 "com.example")
///
/// # 返回
/// - `Project`: aqua schema 模型
pub async fn import_from_db(
    driver: &dyn Driver,
    schema: &str,
    base_package: Option<String>,
) -> Result<Project, DriverError> {
    // 1. 获取所有表
    let table_names = driver.list_tables(schema).await?;

    // 2. 逐表反解
    let mut tables = Vec::new();
    for table_name in &table_names {
        let table = import_table(driver, table_name).await?;
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

/// 导入单个表。
async fn import_table(driver: &dyn Driver, table_name: &str) -> Result<Table, DriverError> {
    // 1. 获取列
    let columns = driver.get_columns(table_name).await?;
    let fields: Vec<Field> = columns.into_iter().map(column_to_field).collect();

    // 2. 获取索引
    let indexes_meta = driver.list_indexes(table_name).await?;
    let indexes: Vec<Index> = indexes_meta.into_iter().map(index_meta_to_index).collect();

    // 3. 构造 Table
    Ok(Table {
        code: table_name.to_uppercase(),
        name: table_name.to_string(),  // 暂无注释,用表名
        group: "default".to_string(),   // 默认分组
        fields,
        indexes: if indexes.is_empty() { None } else { Some(indexes) },
        comment: None,
    })
}

/// ColumnMeta → Field 转换。
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
        enum_ref: None,      // 枚举识别待后续优化
        biz_type: None,
        biz_type_data: None,
        comment: None,
    }
}

/// IndexMeta → Index 转换。
fn index_meta_to_index(idx: IndexMeta) -> Index {
    Index {
        name: Some(idx.name),
        fields: idx.fields.iter().map(|f| f.to_uppercase()).collect(),
        unique: idx.unique,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DataType;

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
        assert_eq!(field.data_type, DataType::Varchar);
        assert_eq!(field.not_null, Some(true));
    }

    #[test]
    fn test_index_meta_to_index() {
        let idx = IndexMeta {
            name: "idx_user_name".to_string(),
            fields: vec!["user_name".to_string(), "status".to_string()],
            unique: true,
        };

        let index = index_meta_to_index(idx);
        assert_eq!(index.name, Some("idx_user_name".to_string()));
        assert_eq!(index.fields, vec!["USER_NAME", "STATUS"]);
        assert_eq!(index.unique, true);
    }
}
