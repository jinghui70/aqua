//! StrConst 生成器 - 生成数据库常量类(表名/字段名)。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/database-consts/`。
//! 规则见 `docs/design.md` §4.2.3。

use crate::schema::Project;
use std::collections::HashSet;

/// StrConst 生成选项。
#[derive(Debug, Clone, Default)]
pub struct StrConstOptions {
    /// 分组过滤(为空则全部表)。有分组 -> 包名 = {basePackage}.{group},类名 DatabaseConstants
    pub group: Option<String>,
}

/// 生成 StrConst Java 常量类。
///
/// 规则(§4.2.3):
/// - 表名 + 字段名都导出
/// - 字段名跨表去重
/// - 范围: 全部表或指定分组
/// - 类名固定 DatabaseConstants;全部表包名 = basePackage,按分组 = basePackage.group
pub fn generate_strconst(project: &Project, options: &StrConstOptions) -> String {
    let full_package = if let Some(g) = &options.group {
        format!("{}.{}", project.base_package, g)
    } else {
        project.base_package.clone()
    };
    let class_name = "DatabaseConstants";
    let mut lines = Vec::new();

    // Javadoc(工具生成,请勿手动修改)
    lines.push("/**".to_string());
    lines.push(" * 工具生成,请勿手动修改".to_string());
    lines.push(" */".to_string());

    // Package 声明
    lines.push(format!("package {};", full_package));
    lines.push(String::new());

    // 类声明
    lines.push(format!("public class {} {{", class_name));

    // 过滤表
    let tables: Vec<_> = project
        .tables
        .iter()
        .filter(|t| options.group.as_ref().map_or(true, |g| &t.group == g))
        .collect();

    // 表名常量
    if !tables.is_empty() {
        lines.push("    // 表名".to_string());
        for table in &tables {
            lines.push(format!(
                "    public static final String {} = \"{}\";",
                table.code, table.code
            ));
        }
    }

    // 字段名常量(跨表去重,保持首次出现顺序)
    let mut seen: HashSet<&str> = HashSet::new();
    let mut field_names: Vec<&str> = Vec::new();
    for table in &tables {
        for field in &table.fields {
            let code = field.code.as_str();
            if seen.insert(code) {
                field_names.push(code);
            }
        }
    }

    if !field_names.is_empty() {
        if !tables.is_empty() {
            lines.push(String::new());
        }
        lines.push("    // 字段名".to_string());
        for name in &field_names {
            lines.push(format!(
                "    public static final String {} = \"{}\";",
                name, name
            ));
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{DataType, Field, Table};

    fn make_project() -> Project {
        Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            biz_types: vec![],
        auto_gen_strategies: vec![],
            groups: vec![],
            tables: vec![
                Table {
                    code: "SYS_USER".to_string(),
                    name: "用户".to_string(),
                    group: "core".to_string(),
                    fields: vec![
                        Field {
                            prop: "id".to_string(),
                            code: "ID".to_string(),
                            name: "主键".to_string(),
                            data_type: DataType::Long,
                            length: None,
                            precision: None,
                            scale: None,
                            biz_type: None,
                            biz_type_data: None,
                            is_key: Some(true),
                            not_null: Some(true),
                            auto_generate: None,
                            default_value: None,
                            enum_ref: None,
                            comment: None,
                        },
                        Field {
                            prop: "userName".to_string(),
                            code: "USER_NAME".to_string(),
                            name: "用户名".to_string(),
                            data_type: DataType::Varchar,
                            length: Some(64),
                            precision: None,
                            scale: None,
                            biz_type: None,
                            biz_type_data: None,
                            is_key: None,
                            not_null: None,
                            auto_generate: None,
                            default_value: None,
                            enum_ref: None,
                            comment: None,
                        },
                    ],
                    indexes: None,
                    comment: None,
                },
                Table {
                    code: "SYS_ROLE".to_string(),
                    name: "角色".to_string(),
                    group: "core".to_string(),
                    fields: vec![
                        Field {
                            prop: "id".to_string(),
                            code: "ID".to_string(),
                            name: "主键".to_string(),
                            data_type: DataType::Long,
                            length: None,
                            precision: None,
                            scale: None,
                            biz_type: None,
                            biz_type_data: None,
                            is_key: Some(true),
                            not_null: Some(true),
                            auto_generate: None,
                            default_value: None,
                            enum_ref: None,
                            comment: None,
                        },
                        Field {
                            prop: "roleName".to_string(),
                            code: "ROLE_NAME".to_string(),
                            name: "角色名".to_string(),
                            data_type: DataType::Varchar,
                            length: Some(64),
                            precision: None,
                            scale: None,
                            biz_type: None,
                            biz_type_data: None,
                            is_key: None,
                            not_null: None,
                            auto_generate: None,
                            default_value: None,
                            enum_ref: None,
                            comment: None,
                        },
                    ],
                    indexes: None,
                    comment: None,
                },
            ],
        }
    }

    #[test]
    fn test_generate_strconst() {
        let project = make_project();
        let result = generate_strconst(&project, &StrConstOptions::default());

        assert!(result.contains("package com.example;"));
        assert!(result.contains("public class DatabaseConstants {"));
        assert!(result.contains("工具生成,请勿手动修改"));
        // 表名
        assert!(result.contains("public static final String SYS_USER = \"SYS_USER\";"));
        assert!(result.contains("public static final String SYS_ROLE = \"SYS_ROLE\";"));
        // 字段名(ID 跨表去重,只出现一次)
        let id_count = result
            .matches("public static final String ID = \"ID\";")
            .count();
        assert_eq!(id_count, 1, "ID 应去重只出现一次");
        assert!(result.contains("public static final String USER_NAME = \"USER_NAME\";"));
        assert!(result.contains("public static final String ROLE_NAME = \"ROLE_NAME\";"));
    }

    #[test]
    fn test_group_filter() {
        let project = make_project();
        let options = StrConstOptions {
            group: Some("core".to_string()),
            ..Default::default()
        };
        let result = generate_strconst(&project, &options);
        assert!(result.contains("package com.example.core;"));
        assert!(result.contains("SYS_USER"));
        assert!(result.contains("SYS_ROLE"));
    }
}
