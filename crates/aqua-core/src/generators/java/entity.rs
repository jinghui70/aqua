//! 实体类生成逻辑。

use super::naming::{snake_to_camel, snake_to_pascal};
use super::types::{get_java_import, map_java_type, JavaOptions};
use crate::schema::{Field, Project, Table};
use std::collections::HashSet;

/// 生成 Java 实体类。
pub fn generate_entity_class(
    project: &Project,
    table: &Table,
    options: &JavaOptions,
) -> Result<String, String> {
    let package = options
        .package
        .clone()
        .unwrap_or_else(|| default_package(project, table));

    let class_name = options
        .class_name
        .clone()
        .unwrap_or_else(|| snake_to_pascal(&table.code));

    let mut output = Vec::new();

    // Package 声明
    output.push(format!("package {};", package));
    output.push(String::new());

    // Import 收集
    let imports = collect_imports(table, options);
    for import in &imports {
        output.push(format!("import {};", import));
    }
    if !imports.is_empty() {
        output.push(String::new());
    }

    // 类注解
    if options.include_comment {
        output.push(javadoc(&table.name, &table.comment, ""));
    }
    output.push(format!("@Table(name = \"{}\")", table.code));
    if options.use_lombok {
        output.push("@Data".to_string());
    }

    // 类定义
    output.push(format!("public class {} {{", class_name));
    output.push(String::new());

    // 字段定义
    for field in &table.fields {
        output.extend(generate_field(field, options));
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

/// 默认 package: {basePackage}.{group}.entity
fn default_package(project: &Project, table: &Table) -> String {
    format!(
        "{}.{}.entity",
        project.base_package,
        table.group.to_lowercase()
    )
}

/// 收集需要的 imports。
fn collect_imports(table: &Table, options: &JavaOptions) -> Vec<String> {
    let mut imports = HashSet::new();

    // rainbow-dbaccess 注解
    imports.insert("io.github.rainbow.dbaccess.annotation.Table".to_string());
    imports.insert("io.github.rainbow.dbaccess.annotation.Id".to_string());
    imports.insert("io.github.rainbow.dbaccess.annotation.Column".to_string());

    // Lombok
    if options.use_lombok {
        imports.insert("lombok.Data".to_string());
    }

    // 字段类型
    for field in &table.fields {
        if let Some(import) = get_java_import(field.data_type) {
            imports.insert(import.to_string());
        }
    }

    let mut sorted: Vec<_> = imports.into_iter().collect();
    sorted.sort();
    sorted
}

/// 生成字段定义。
fn generate_field(field: &Field, options: &JavaOptions) -> Vec<String> {
    let mut lines = Vec::new();

    // Javadoc 注释(中文名 + 备注)
    if options.include_comment {
        lines.push(javadoc(&field.name, &field.comment, "    "));
    }

    // 字段注解(顺序: @Id -> @GeneratedValue -> @Column,对齐 legacy)
    if field.is_key.unwrap_or(false) {
        lines.push("    @Id".to_string());
    }

    // @GeneratedValue(自动生成字段,enabled=false 不输出)
    // 参数等于默认值即省略:strategy="default"、timing=INSERT、param=空
    if let Some(ag) = &field.auto_generate {
        if ag.enabled {
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
            lines.push(format!("    @GeneratedValue({})", parts.join(", ")));
        }
    }

    // @Column (非标准命名时)
    let prop = &field.prop;
    let expected_prop = snake_to_camel(&field.code);
    if prop != &expected_prop {
        lines.push(format!("    @Column(name = \"{}\")", field.code));
    }

    // 字段声明
    let java_type = map_java_type(field.data_type);
    lines.push(format!("    private {} {};", java_type, prop));
    lines.push(String::new());

    lines
}

/// 生成 getter/setter (非 Lombok 时)。
fn generate_getter_setter(field: &Field) -> Vec<String> {
    let mut lines = Vec::new();

    let prop = &field.prop;
    let java_type = map_java_type(field.data_type);
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
