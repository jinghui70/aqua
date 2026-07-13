//! 枚举类 Java 生成(§3.5)。
//!
//! 全局枚举(EnumDefine):
//! - hasCode=true: `MALE("M","男")` + implements CodeEnum + code/name 字段/getter
//! - hasCode=false: 普通枚举 `MALE // 男`

use crate::schema::{EnumDefine, EnumValue, Project};

/// 单个枚举值行(hasCode 分支)。
fn enum_value_line(value: &EnumValue, has_code: bool) -> String {
    match (has_code, &value.code) {
        (true, Some(code)) => format!("  {}(\"{}\", \"{}\")", value.id, code, value.name),
        _ => format!("  {} // {}", value.id, value.name),
    }
}

/// 生成全局枚举 Java 类。
///
/// package = {basePackage}.{enum_def.package}
pub fn generate_global_enum_class(project: &Project, enum_def: &EnumDefine) -> String {
    let pkg = format!("{}.{}", project.base_package, enum_def.package);
    let class_name = &enum_def.code;
    let has_code = enum_def.has_code.unwrap_or(false);

    let mut lines = vec![format!("package {};", pkg), String::new()];

    // Javadoc + 类声明
    lines.push("/**".to_string());
    if has_code {
        lines.push(format!(" * {}(CodeEnum 派生,存储 code)", enum_def.name));
    } else {
        lines.push(format!(" * {}(普通枚举,存储 id)", enum_def.name));
    }
    lines.push(" */".to_string());
    let impl_clause = if has_code { " implements CodeEnum" } else { "" };
    lines.push(format!("public enum {}{} {{", class_name, impl_clause));

    // 枚举值
    let value_lines: Vec<String> = enum_def
        .values
        .iter()
        .map(|v| enum_value_line(v, has_code))
        .collect();
    lines.push(format!("{};", value_lines.join(",\n")));

    // hasCode: 字段 + 构造 + getter
    if has_code {
        lines.push(String::new());
        lines.push("  private final String code;".to_string());
        lines.push("  private final String name;".to_string());
        lines.push(String::new());
        lines.push(format!("  {}(String code, String name) {{", class_name));
        lines.push("    this.code = code;".to_string());
        lines.push("    this.name = name;".to_string());
        lines.push("  }".to_string());
        lines.push(String::new());
        lines.push("  @Override".to_string());
        lines.push("  public String getCode() {".to_string());
        lines.push("    return code;".to_string());
        lines.push("  }".to_string());
        lines.push(String::new());
        lines.push("  public String getName() {".to_string());
        lines.push("    return name;".to_string());
        lines.push("  }".to_string());
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::EnumValue;

    fn make_project(enum_def: EnumDefine) -> Project {
        Project {
            version: "1.0.0".to_string(),
            name: None,
            base_package: "com.example".to_string(),
            biz_types: vec![],
            enums: vec![enum_def],
            groups: vec![],
            tables: vec![],
        }
    }

    #[test]
    fn test_has_code_enum() {
        let def = EnumDefine {
            code: "EnumGender".to_string(),
            name: "性别".to_string(),
            package: "enum".to_string(),
            has_code: Some(true),
            values: vec![
                EnumValue {
                    id: "MALE".to_string(),
                    name: "男".to_string(),
                    code: Some("M".to_string()),
                    color: None,
                },
                EnumValue {
                    id: "FEMALE".to_string(),
                    name: "女".to_string(),
                    code: Some("F".to_string()),
                    color: None,
                },
            ],
        };
        let project = make_project(def.clone());
        let java = generate_global_enum_class(&project, &def);

        assert!(java.contains("package com.example.enum;"));
        assert!(java.contains("public enum EnumGender implements CodeEnum {"));
        assert!(java.contains("MALE(\"M\", \"男\")"));
        assert!(java.contains("FEMALE(\"F\", \"女\")"));
        assert!(java.contains("public String getCode()"));
        assert!(java.contains("private final String code;"));
    }

    #[test]
    fn test_no_code_enum() {
        let def = EnumDefine {
            code: "EnumStatus".to_string(),
            name: "状态".to_string(),
            package: "enum".to_string(),
            has_code: Some(false),
            values: vec![
                EnumValue {
                    id: "ACTIVE".to_string(),
                    name: "启用".to_string(),
                    code: None,
                    color: None,
                },
                EnumValue {
                    id: "DISABLED".to_string(),
                    name: "禁用".to_string(),
                    code: None,
                    color: None,
                },
            ],
        };
        let project = make_project(def.clone());
        let java = generate_global_enum_class(&project, &def);

        assert!(java.contains("public enum EnumStatus {"));
        assert!(!java.contains("implements CodeEnum"));
        assert!(java.contains("ACTIVE // 启用"));
        assert!(java.contains("DISABLED // 禁用"));
        assert!(!java.contains("getCode"));
    }
}
