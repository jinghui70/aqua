//! 枚举类生成 - 普通 / CodeEnum 两形态。
//!
//! 形态由 `enum.hasCode` 决定(§3.5,用户给定):
//! - hasCode=false/无: 数据库存 id,生成普通枚举,项后 // {name} 注释
//! - hasCode=true:     数据库存 code,生成 implements CodeEnum,单参构造 + code() 覆写

use crate::schema::EnumDefine;

/// rainbow-dbaccess CodeEnum 接口全限定名(已确认 6.3.3),生成时用 import 引入。
pub const CODE_ENUM: &str = "io.github.jinghui70.rainbow.dbaccess.object.CodeEnum";

/// 生成独立枚举类文件内容(含 package 声明,若 options.package 存在)。
pub fn generate_enum_class(def: &EnumDefine, package: Option<&str>) -> String {
    let e = &def.r#enum;
    let class_name = e
        .class_name
        .clone()
        .unwrap_or_else(|| crate::generators::java::naming::prop_to_pascal(&def.field_prop));
    let has_code = e.has_code.unwrap_or(false);
    let mut lines = Vec::new();

    if let Some(pkg) = package {
        lines.push(format!("package {};", pkg));
        lines.push(String::new());
    }

    // CodeEnum 用 import 引入(不用全限定名),声明处短名
    if has_code {
        lines.push(format!("import {};", CODE_ENUM));
        lines.push(String::new());
    }

    // 类 Javadoc(枚举名)
    lines.push(format!("/** {} */", e.name));
    let decl = if has_code {
        format!("public enum {} implements CodeEnum {{", class_name)
    } else {
        format!("public enum {} {{", class_name)
    };
    lines.push(decl);

    let last = e.values.len().saturating_sub(1);
    if has_code {
        // 枚举项: MALE("M"), // 男  末项以 ; 结尾
        for (i, v) in e.values.iter().enumerate() {
            let sep = if i == last { ";" } else { "," };
            lines.push(format!(
                "    {}(\"{}\"){} // {}",
                v.id,
                v.code.as_deref().unwrap_or(""),
                sep,
                v.name
            ));
        }
        lines.push(String::new());
        lines.push("    private final String code;".to_string());
        lines.push(String::new());
        lines.push(format!(
            "    {}(String code) {{ this.code = code; }}",
            class_name
        ));
        lines.push(String::new());
        lines.push("    @Override".to_string());
        lines.push("    public String code() { return code; }".to_string());
        lines.push("}".to_string());
    } else {
        // 普通枚举: MALE, // 男  末项以 ; 结尾
        for (i, v) in e.values.iter().enumerate() {
            let sep = if i == last { ";" } else { "," };
            lines.push(format!("    {}{} // {}", v.id, sep, v.name));
        }
        lines.push("}".to_string());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{EnumColor, EnumValue, InlineEnum};

    fn def(has_code: bool) -> EnumDefine {
        let values = if has_code {
            vec![
                EnumValue {
                    id: "MALE".into(),
                    name: "男".into(),
                    code: Some("M".into()),
                    color: Some(EnumColor::Blue),
                },
                EnumValue {
                    id: "FEMALE".into(),
                    name: "女".into(),
                    code: Some("F".into()),
                    color: Some(EnumColor::Red),
                },
            ]
        } else {
            vec![EnumValue {
                id: "ACTIVE".into(),
                name: "启用".into(),
                code: None,
                color: Some(EnumColor::Success),
            }]
        };
        EnumDefine {
            table_code: "T".into(),
            field_prop: "gender".into(),
            // field 字段此处不影响枚举类输出
            field: crate::schema::Field {
                prop: "gender".into(),
                code: "GENDER".into(),
                name: "性别".into(),
                data_type: crate::schema::DataType::Varchar,
                length: Some(8),
                precision: None,
                scale: None,
                is_key: None,
                not_null: None,
                default_value: None,
                auto_generate: None,
                enum_ref: Some(InlineEnum {
                    name: "性别".into(),
                    has_code: Some(has_code),
                    class_name: None,
                    r#ref: None,
                    values: values.clone(),
                }),
                biz_type: None,
                biz_type_data: None,
                comment: None,
            },
            r#enum: InlineEnum {
                name: "性别".into(),
                has_code: Some(has_code),
                class_name: None,
                r#ref: None,
                values,
            },
        }
    }

    #[test]
    fn plain_enum_has_no_code() {
        let out = generate_enum_class(&def(false), None);
        assert!(out.contains("public enum Gender {"));
        // 单值枚举即首项/末项,以 ; 结尾
        assert!(out.contains("ACTIVE; // 启用"));
        assert!(!out.contains("implements"));
        assert!(!out.contains("code()"));
    }

    #[test]
    fn code_enum_implements_codeenum() {
        let out = generate_enum_class(&def(true), None);
        assert!(out.contains("import io.github.jinghui70.rainbow.dbaccess.object.CodeEnum;"));
        assert!(out.contains("implements CodeEnum {"));
        assert!(out.contains("MALE(\"M\"), // 男"));
        assert!(out.contains("FEMALE(\"F\"); // 女"));
        assert!(out.contains("private final String code;"));
        assert!(out.contains("public String code() { return code; }"));
    }

    #[test]
    fn code_enum_last_terminator_semicolon_first_comma() {
        let out = generate_enum_class(&def(true), None);
        // 第一项逗号, 末项分号
        assert!(out.contains("MALE(\"M\"),"));
        assert!(out.contains("FEMALE(\"F\");"));
    }

    #[test]
    fn package_declaration_included_when_given() {
        let out = generate_enum_class(&def(false), Some("com.x.core.entity"));
        assert!(out.starts_with("package com.x.core.entity;"));
    }
}
