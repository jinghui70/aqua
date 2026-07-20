//! 从数据库导入 schema 核心逻辑。

use crate::driver::{ColumnMeta, Driver, DriverError, IndexMeta, TableInfo};
use crate::generators::java::naming::snake_to_camel;
use crate::schema::{Direction, Field, Index, IndexField, Project, Table};

/// 从数据库导入 schema,生成 Project。
///
/// 仅反解 `tables` 指定的表(用户在导入向导选中的表),避免整库反解的无效 spawn 开销。
///
/// # 参数
/// - `driver`: 数据库驱动
/// - `tables`: 要导入的表名列表(用户选中)
/// - `base_package`: 基础包名(默认 "com.example")
///
/// # 返回
/// - `Project`: aqua schema 模型
pub async fn import_from_db(
    driver: &dyn Driver,
    tables: &[TableInfo],
    base_package: Option<String>,
) -> Result<Project, DriverError> {
    // 逐表反解(仅选中表,表注释从 listTables 复用,不另 spawn)
    let mut result = Vec::new();
    for table in tables {
        let t = import_table(driver, table).await?;
        result.push(t);
    }

    Ok(Project {
        version: "1.0.0".to_string(),
        name: None,
        base_package: base_package.unwrap_or_else(|| "com.example".to_string()),
        tables: result,
        enums: vec![],
        biz_types: vec![],
        groups: vec![],
    })
}

/// 导入单个表。
async fn import_table(driver: &dyn Driver, table: &TableInfo) -> Result<Table, DriverError> {
    // 1. 获取列
    let columns = driver.get_columns(&table.name).await?;
    let fields: Vec<Field> = columns.into_iter().map(column_to_field).collect();

    // 2. 获取索引
    let indexes_meta = driver.list_indexes(&table.name).await?;
    let indexes: Vec<Index> = indexes_meta.into_iter().map(index_meta_to_index).collect();

    // 3. 构造 Table(表注释作为中文名 name;无注释时回退表名)
    Ok(Table {
        code: table.name.to_uppercase(),
        name: table
            .comment
            .clone()
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| table.name.clone()),
        group: "default".to_string(),
        fields,
        indexes: if indexes.is_empty() {
            None
        } else {
            Some(indexes)
        },
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
        auto_generate: None, // 导入时无法推断,需人工配置
        default_value: col.default_value,
        enum_ref: None, // 枚举识别待后续优化
        biz_type: None,
        biz_type_data: None,
        comment: None,
    }
}

/// IndexMeta → Index 转换。
fn index_meta_to_index(idx: IndexMeta) -> Index {
    Index {
        name: Some(idx.name),
        fields: idx
            .fields
            .iter()
            .map(|f| IndexField {
                code: f.to_uppercase(),
                direction: Direction::Asc,
            })
            .collect(),
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
        assert_eq!(
            index.fields,
            vec![
                IndexField {
                    code: "USER_NAME".to_string(),
                    direction: Direction::Asc,
                },
                IndexField {
                    code: "STATUS".to_string(),
                    direction: Direction::Asc,
                },
            ]
        );
        assert!(index.unique);
    }
}
