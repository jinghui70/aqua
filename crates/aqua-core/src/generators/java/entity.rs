//! 实体类生成逻辑。

use super::enumeration::generate_enum_class;
use super::naming::{prop_to_pascal, snake_to_camel, snake_to_pascal};
use super::types::{get_java_import, map_java_type, JavaOptions};
use super::JavaFile;
use crate::schema::{DataType, EnumDefine, Field, Project, Table};
use std::collections::{HashMap, HashSet};

/// 引用枚举的包名:优先取定义表持久化的 javaPackage(用户在 JavaTab 改过则记住),
/// 无则按默认规则 {basePackage}.{group}.entity(与 JavaTab 预填一致)。
fn enum_package(project: &Project, def_table_code: &str) -> Option<String> {
    let t = project.tables.iter().find(|t| t.code == def_table_code)?;
    if let Some(pkg) = t.java_package.as_deref().filter(|p| !p.trim().is_empty()) {
        return Some(pkg.to_string());
    }
    let base = project.base_package.trim();
    let group = t.group.to_lowercase();
    let suffix = if group.is_empty() {
        "entity".to_string()
    } else {
        format!("{group}.entity")
    };
    Some(if base.is_empty() {
        suffix
    } else {
        format!("{base}.{suffix}")
    })
}

/// 引用字段的枚举类名(解析定义方 className/prop 派生);目标缺失返回 None。
fn ref_enum_class(field: &Field, defs: &HashMap<(String, String), EnumDefine>) -> Option<String> {
    let r = field.enum_ref.as_ref()?.r#ref.as_ref()?;
    let def = defs.get(&(r.code.clone(), r.prop.clone()))?;
    Some(
        def.r#enum
            .class_name
            .clone()
            .unwrap_or_else(|| prop_to_pascal(&def.field_prop)),
    )
}

/// 生成实体文件 + 本表定义方的枚举文件(Vec 有序:实体首,枚举随后)。
pub fn generate_entity_files(
    project: &Project,
    table: &Table,
    options: &JavaOptions,
) -> Result<Vec<JavaFile>, String> {
    let defs = project.enum_defs();

    let entity_class = generate_entity_class(project, table, options, &defs)?;
    let class_name = options
        .class_name
        .clone()
        .unwrap_or_else(|| snake_to_pascal(&table.code));
    let mut files = vec![JavaFile {
        path: format!("{class_name}.java"),
        content: entity_class,
    }];

    // 本表定义方枚举 -> 独立枚举文件(引用方不产文件)
    for field in table.fields.iter() {
        let Some(e) = &field.enum_ref else { continue };
        if e.r#ref.is_some() {
            continue; // 引用方
        }
        if let Some(def) = defs.get(&(table.code.clone(), field.prop.clone())) {
            let class_name = e
                .class_name
                .clone()
                .unwrap_or_else(|| prop_to_pascal(&field.prop));
            files.push(JavaFile {
                path: format!("{class_name}.java"),
                content: generate_enum_class(def, options.package.as_deref()),
            });
        }
    }

    Ok(files)
}

/// 生成 Java 实体类(单个类文本)。
pub fn generate_entity_class(
    project: &Project,
    table: &Table,
    options: &JavaOptions,
    defs: &HashMap<(String, String), EnumDefine>,
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

    // Import 收集(含引用枚举跨包 import)
    let imports = collect_imports(project, table, options, defs, need_table_anno);
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
        output.extend(generate_field(field, defs));
    }

    // getter/setter (非 Lombok 时)
    if !options.use_lombok {
        for field in &table.fields {
            output.extend(generate_getter_setter(field, defs));
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

/// 收集需要的 imports(仅 import 实际用到的注解 + 引用枚举类)。
fn collect_imports(
    project: &Project,
    table: &Table,
    options: &JavaOptions,
    defs: &HashMap<(String, String), EnumDefine>,
    need_table_anno: bool,
) -> Vec<String> {
    const ANNO: &str = "io.github.jinghui70.rainbow.dbaccess.annotation";
    let mut imports = HashSet::new();

    // @Table(仅在类名不匹配表名时)
    if need_table_anno {
        imports.insert(format!("{ANNO}.Table"));
    }

    // 引用枚举:枚举类由定义方表生成(其默认包 {base}.{group}.entity),
    // 与实体包不同(跨组)时 import;同包无需。
    for field in &table.fields {
        let Some(r) = field.enum_ref.as_ref().and_then(|e| e.r#ref.as_ref()) else {
            continue;
        };
        let (Some(cls), Some(pkg)) = (ref_enum_class(field, defs), enum_package(project, &r.code))
        else {
            continue;
        };
        if options.package.as_deref() != Some(pkg.as_str()) {
            imports.insert(format!("{pkg}.{cls}"));
        }
    }

    // 扫描字段,按实际使用收集注解 import
    let mut use_id = false;
    let mut use_column = false;
    let mut use_generated_value = false;
    let mut use_generate_timing = false;
    for field in &table.fields {
        if field.is_key.unwrap_or(false) {
            use_id = true;
        }
        if let Some(ag) = &field.auto_generate {
            use_generated_value = true;
            // timing=INSERT_UPDATE 输出 GenerationTiming 枚举引用,需 import
            if ag.timing == crate::schema::GenerationTiming::InsertUpdate {
                use_generate_timing = true;
            }
        }
        // @Column: 非标准命名 or Clob/Blob(sqlType=Types.CLOB/BLOB)
        let expected_prop = snake_to_camel(&field.code);
        if field.prop != expected_prop || matches!(field.data_type, DataType::Clob | DataType::Blob)
        {
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
    if use_generate_timing {
        imports.insert(format!("{ANNO}.GenerationTiming"));
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

/// 枚举字段的 Java 类型(枚举类名)。defs 为全项目定义索引。
/// 非枚举字段返回 None。
fn enum_type_name(field: &Field, defs: &HashMap<(String, String), EnumDefine>) -> Option<String> {
    let e = field.enum_ref.as_ref()?;
    if let Some(r) = &e.r#ref {
        // 引用方:解析目标定义方类名
        defs.get(&(r.code.clone(), r.prop.clone())).map(|d| {
            d.r#enum
                .class_name
                .clone()
                .unwrap_or_else(|| prop_to_pascal(&d.field_prop))
        })
    } else {
        // 定义方:自身类名
        Some(
            e.class_name
                .clone()
                .unwrap_or_else(|| prop_to_pascal(&field.prop)),
        )
    }
}

/// 字段 Java 类型:notNull=true -> 基本类型(boolean/int/long),可空 -> 包装类型(Boolean/Integer/Long)。
/// bizType=Bool 不限物理类型(TINYINT/INT/VARCHAR),统一按此规则映射 boolean/Boolean。
/// 枚举字段 -> 枚举类名。
fn java_type_for(field: &Field, defs: &HashMap<(String, String), EnumDefine>) -> String {
    if let Some(enum_type) = enum_type_name(field, defs) {
        return enum_type;
    }
    let primitive = field.not_null.unwrap_or(false);
    if field.biz_type.as_deref() == Some("Bool") {
        if primitive {
            "boolean".to_string()
        } else {
            "Boolean".to_string()
        }
    } else {
        match field.data_type {
            DataType::Tinyint | DataType::Int => {
                if primitive {
                    "int".to_string()
                } else {
                    "Integer".to_string()
                }
            }
            DataType::Long => {
                if primitive {
                    "long".to_string()
                } else {
                    "Long".to_string()
                }
            }
            _ => map_java_type(field.data_type).to_string(),
        }
    }
}

/// 生成字段定义。
fn generate_field(field: &Field, defs: &HashMap<(String, String), EnumDefine>) -> Vec<String> {
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
        if ag.timing == crate::schema::GenerationTiming::InsertUpdate {
            parts.push("timing = GenerationTiming.INSERT_UPDATE".to_string());
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
    let java_type = java_type_for(field, defs);
    lines.push(format!("    private {} {};", java_type, prop));
    lines.push(String::new());

    lines
}

/// 生成 getter/setter (非 Lombok 时)。
fn generate_getter_setter(
    field: &Field,
    defs: &HashMap<(String, String), EnumDefine>,
) -> Vec<String> {
    let mut lines = Vec::new();

    let prop = &field.prop;
    let java_type = java_type_for(field, defs);
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
