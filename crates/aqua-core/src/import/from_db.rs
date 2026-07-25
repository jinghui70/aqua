//! 从数据库导入 schema 核心逻辑。

use crate::driver::{ColumnMeta, Driver, DriverError, IndexMeta, TableInfo, TableMeta};
use crate::generators::java::naming::snake_to_camel;
use crate::schema::{Direction, Field, Index, IndexField, Project, Table};
use std::collections::HashMap;

/// 从数据库导入 schema,生成 Project。
///
/// 仅反解 `tables` 指定的表(用户在导入向导选中的表)。通过 `driver.import_tables` 批量反解:
/// JDBC 一次 spawn 拿全部表元数据,消除逐表 2N 次 JVM 冷启动;native 走连接池,无额外开销。
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
    // 一次批量反解全部选中表(JDBC 单次 spawn)
    let names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
    let metas = driver.import_tables(&names).await?;

    // name → 列/索引元数据,按输入表顺序组装(表注释复用 TableInfo,不另查)
    let mut meta_map: HashMap<String, TableMeta> =
        metas.into_iter().map(|m| (m.name.clone(), m)).collect();

    let mut result = Vec::with_capacity(tables.len());
    for table in tables {
        let meta = meta_map.remove(&table.name).ok_or_else(|| {
            DriverError::QueryFailed(format!("导入结果缺少表 {} 的元数据", table.name))
        })?;
        result.push(build_table(table, meta));
    }

    Ok(Project {
        version: "1.0.0".to_string(),
        name: None,
        base_package: base_package.unwrap_or_else(|| "com.example".to_string()),
        tables: result,
        biz_types: vec![],
        auto_gen_strategies: vec![],
        groups: vec![],
    })
}

/// 由批量反解的元数据 + 表信息组装单个 Table(纯组装,不触驱动)。
fn build_table(table: &TableInfo, meta: TableMeta) -> Table {
    let fields: Vec<Field> = meta.columns.into_iter().map(column_to_field).collect();
    let indexes: Vec<Index> = meta.indexes.into_iter().map(index_meta_to_index).collect();

    // 表注释作为中文名 name;无注释时回退表名
    Table {
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
    }
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
