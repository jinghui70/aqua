//! StrConst 生成器 - 生成数据库常量类(表名/字段名)。
//!
//! 移植自 `~/work/aqua-legacy/packages/core/src/generators/database-consts/`。
//! 规则见 `docs/design.md` §4.2.3。

use crate::schema::Project;
use std::collections::HashSet;

/// StrConst 生成选项。
#[derive(Debug, Clone)]
pub struct StrConstOptions {
    /// 包名后缀(默认 "const"),完整包 = {basePackage}.{suffix}
    pub package_suffix: String,
    /// 类名(默认 "StrConst")
    pub class_name: String,
    /// 分组过滤(为空则全部表)
    pub group: Option<String>,
}

impl Default for StrConstOptions {
    fn default() -> Self {
        Self {
            package_suffix: "const".to_string(),
            class_name: "DatabaseConstants".to_string(),
            group: None,
        }
    }
}

/// 生成 StrConst Java 常量类。
///
/// 规则(§4.2.3):
/// - 表名 + 字段名都导出
/// - 字段名跨表去重
/// - 范围: 全部表或指定分组
pub fn generate_strconst(project: &Project, options: &StrConstOptions) -> String {
    let full_package = format!("{}.{}", project.base_package, options.package_suffix);
    let mut lines = Vec::new();

    // Package 声明
    lines.push(format!("package {};", full_package));
    lines.push(String::new());

    // 类声明
    lines.push(format!("public class {} {{", options.class_name));

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

        assert!(result.contains("package com.example.const;"));
        assert!(result.contains("public class DatabaseConstants {"));
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
        assert!(result.contains("SYS_USER"));
        assert!(result.contains("SYS_ROLE"));
    }

    #[test]
    fn test_custom_class_name() {
        let project = make_project();
        let options = StrConstOptions {
            class_name: "DbConst".to_string(),
            package_suffix: "constants".to_string(),
            ..Default::default()
        };
        let result = generate_strconst(&project, &options);
        assert!(result.contains("package com.example.constants;"));
        assert!(result.contains("public class DbConst {"));
    }
}
