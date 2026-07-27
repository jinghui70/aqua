//! 实体类生成逻辑。

use super::naming::{snake_to_camel, snake_to_pascal};
use super::types::{get_java_import, map_java_type, JavaOptions};
use crate::schema::{DataType, Field, Table};
use std::collections::HashSet;

/// 生成 Java 实体类。
pub fn generate_entity_class(
    table: &Table,
    options: &JavaOptions,
) -> Result<String, String> {
    let class_name = options
        .class_name
        .clone()
        .unwrap_or_else(|| snake_to_pascal(&table.code));

    // @Table 省略判据:类名恰好能反推表名(SysUser↔SYS_USER)时约定生效,省略;
    // 否则(自定义类名不匹配)必须显式 @Table 指定表名。与字段 @Column 逻辑对称。
    let need_table_anno = class_name != snake_to_pascal(&table.code);

    let mut output = Vec::new();

    // Package 声明(仅当指定了 package 时生成)
    if let Some(ref package) = options.package {
        output.push(format!("package {};", package));
        output.push(String::new());
    }

    // Import 收集
    let imports = collect_imports(table, options, need_table_anno);
    for import in &imports {
        output.push(format!("import {};", import));
    }
    if !imports.is_empty() {
        output.push(String::new());
    }

    // 类注解(Javadoc 注释始终生成:表名/备注作为文档)
    output.push(javadoc(&table.name, &table.comment, ""));
    if need_table_anno {
        output.push(format!("@Table(name = \"{}\")", table.code));
    }
    if options.use_lombok {
        output.push("@Data".to_string());
    }

    // 类定义
    output.push(format!("public class {} {{", class_name));
    output.push(String::new());

    // 字段定义
    for field in &table.fields {
        output.extend(generate_field(field));
    }

    // getter/setter (非 Lombok 时)
    if !options.use_lombok {
        for field in &table.fields {
            output.extend(generate_getter_setter(field));
        }
    }

    output.push("}".to_string());

    Ok(output.join("\n"))
}

/// 生成 Javadoc 单行注释: `/** 中文名 - 备注 */`,indent 为前置缩进。
fn javadoc(name: &str, comment: &Option<String>, indent: &str) -> String {
    match comment {
        Some(c) if !c.is_empty() => format!("{}/** {} - {} */", indent, name, c),
        _ => format!("{}/** {} */", indent, name),
    }
}

/// 收集需要的 imports(仅 import 实际用到的注解)。
fn collect_imports(table: &Table, options: &JavaOptions, need_table_anno: bool) -> Vec<String> {
    const ANNO: &str = "io.github.jinghui70.rainbow.dbaccess.annotation";
    let mut imports = HashSet::new();

    // @Table(仅在类名不匹配表名时)
    if need_table_anno {
        imports.insert(format!("{ANNO}.Table"));
    }

    // 扫描字段,按实际使用收集注解 import
    let mut use_id = false;
    let mut use_column = false;
    let mut use_generated_value = false;
    for field in &table.fields {
        if field.is_key.unwrap_or(false) {
            use_id = true;
        }
        if field.auto_generate.is_some() {
            use_generated_value = true;
        }
        // @Column: 非标准命名 or Clob/Blob(sqlType=Types.CLOB/BLOB)
        let expected_prop = snake_to_camel(&field.code);
        if field.prop != expected_prop || matches!(field.data_type, DataType::Clob | DataType::Blob) {
            use_column = true;
        }
    }
    if use_id {
        imports.insert(format!("{ANNO}.Id"));
    }
    if use_column {
        imports.insert(format!("{ANNO}.Column"));
    }
    if use_generated_value {
        imports.insert(format!("{ANNO}.GeneratedValue"));
    }

    // Lombok
    if options.use_lombok {
        imports.insert("lombok.Data".to_string());
    }

    // 字段类型 + Clob/Blob 的 java.sql.Types
    for field in &table.fields {
        if let Some(import) = get_java_import(field.data_type) {
            imports.insert(import.to_string());
        }
        if matches!(field.data_type, DataType::Clob | DataType::Blob) {
            imports.insert("java.sql.Types".to_string());
        }
    }

    let mut sorted: Vec<_> = imports.into_iter().collect();
    sorted.sort();
    sorted
}

/// 字段 Java 类型:bizType=Bool -> boolean(基本类型),否则按 data_type 映射
fn java_type_for(field: &Field) -> &'static str {
    if field.biz_type.as_deref() == Some("Bool") {
        "boolean"
    } else {
        map_java_type(field.data_type)
    }
}

/// 生成字段定义。
fn generate_field(field: &Field) -> Vec<String> {
    let mut lines = Vec::new();

    // Javadoc 注释(中文名 + 备注,始终生成)
    lines.push(javadoc(&field.name, &field.comment, "    "));

    // 字段注解(顺序: @Id -> @GeneratedValue -> @Column,对齐 legacy)
    if field.is_key.unwrap_or(false) {
        lines.push("    @Id".to_string());
    }

    // @GeneratedValue(autoGenerate Some 即启用;参数等于默认值省略)
    if let Some(ag) = &field.auto_generate {
        let mut parts = Vec::new();
        if ag.strategy != "default" {
            parts.push(format!("strategy = \"{}\"", ag.strategy));
        }
        if let Some(param) = &ag.param {
            if !param.is_empty() {
                parts.push(format!("param = \"{}\"", param));
            }
        }
        if ag.timing == crate::schema::GenerateTiming::InsertUpdate {
            parts.push("timing = \"INSERT_UPDATE\"".to_string());
        }
        if parts.is_empty() {
            lines.push("    @GeneratedValue".to_string());
        } else {
            lines.push(format!("    @GeneratedValue({})", parts.join(", ")));
        }
    }

    // @Column (非标准命名 or Clob/Blob 加 sqlType)
    let prop = &field.prop;
    let expected_prop = snake_to_camel(&field.code);
    // Clob->Types.CLOB, Blob->Types.BLOB(此前误把 Clob 也写成 BLOB)
    let sql_type = match field.data_type {
        DataType::Clob => Some("Types.CLOB"),
        DataType::Blob => Some("Types.BLOB"),
        _ => None,
    };
    let mut column_parts: Vec<String> = Vec::new();
    if prop != &expected_prop {
        column_parts.push(format!("name = \"{}\"", field.code));
    }
    if let Some(st) = sql_type {
        column_parts.push(format!("sqlType = {}", st));
    }
    if !column_parts.is_empty() {
        lines.push(format!("    @Column({})", column_parts.join(", ")));
    }

    // 字段声明
    let java_type = java_type_for(field);
    lines.push(format!("    private {} {};", java_type, prop));
    lines.push(String::new());

    lines
}

/// 生成 getter/setter (非 Lombok 时)。
fn generate_getter_setter(field: &Field) -> Vec<String> {
    let mut lines = Vec::new();

    let prop = &field.prop;
    let java_type = java_type_for(field);
    let capitalized = if prop.is_empty() {
        String::new()
    } else {
        let mut chars = prop.chars();
        chars.next().unwrap().to_uppercase().to_string() + chars.as_str()
    };

    // getter
    lines.push(format!("    public {} get{}() {{", java_type, capitalized));
    lines.push(format!("        return {};", prop));
    lines.push("    }".to_string());
    lines.push(String::new());

    // setter
    lines.push(format!(
        "    public void set{}({} {}) {{",
        capitalized, java_type, prop
    ));
    lines.push(format!("        this.{} = {};", prop, prop));
    lines.push("    }".to_string());
    lines.push(String::new());

    lines
}
