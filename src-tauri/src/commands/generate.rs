//! generate 命令实现(CLI + Tauri commands)。

use aqua_core::generators::ddl::{generate_ddl, DdlOptions, Dialect};
use aqua_core::generators::java::{generate_java_entity, JavaOptions};
use aqua_core::schema::{parse_project, Project};
use std::error::Error;
use std::fs;

/// CLI generate 命令处理。
pub fn handle_generate(
    type_: String,
    input: String,
    dialect: Option<String>,
    table: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // 1. 读取 input JSON
    let json_str = fs::read_to_string(&input)?;
    let value: serde_json::Value = serde_json::from_str(&json_str)?;
    let project = parse_project(value)?;

    // 2. 根据 type 调用 generator
    let result = match type_.as_str() {
        "ddl" => {
            let dialect_str = dialect.unwrap_or_else(|| "mysql".to_string());
            let dialect = Dialect::parse(&dialect_str)
                .ok_or_else(|| format!("Invalid dialect: {}", dialect_str))?;

            generate_ddl(
                &project,
                &DdlOptions {
                    dialect,
                    ..Default::default()
                },
            )
        }
        "java" => {
            let table_name = table.ok_or("--table required for java")?;
            generate_java_entity(&project, &table_name, &JavaOptions::default())?
        }
        _ => {
            return Err(format!("Unsupported type: {}", type_).into());
        }
    };

    // 3. 输出
    if let Some(out_path) = output {
        fs::write(out_path, result)?;
    } else {
        println!("{}", result);
    }

    Ok(())
}

/// Tauri command: 生成 DDL。
#[tauri::command]
pub async fn generate_ddl_command(
    project: Project,
    dialect: String,
    tables: Option<Vec<String>>,
    group: Option<String>,
) -> Result<String, String> {
    let dialect = Dialect::parse(&dialect).ok_or_else(|| format!("不支持的方言: {}", dialect))?;

    Ok(generate_ddl(
        &project,
        &DdlOptions {
            dialect,
            tables,
            group,
        },
    ))
}

/// Tauri command: 生成 Java 实体类(支持配置: 包名/类名/Lombok/注释)。
#[tauri::command]
pub async fn generate_java_command(
    project: Project,
    table: String,
    use_lombok: Option<bool>,
    include_comment: Option<bool>,
    package: Option<String>,
    class_name: Option<String>,
) -> Result<String, String> {
    let options = JavaOptions {
        use_lombok: use_lombok.unwrap_or(true),
        include_comment: include_comment.unwrap_or(true),
        package,
        class_name,
    };
    generate_java_entity(&project, &table, &options)
}

/// Tauri command: 生成前端 JSON(json-ui 兼容,单表)。
#[tauri::command]
pub async fn generate_frontend_json_command(
    project: Project,
    table: String,
) -> Result<String, String> {
    use aqua_core::generators::frontend_json::{generate_frontend_json, FrontendJsonOptions};
    Ok(generate_frontend_json(
        &project,
        &FrontendJsonOptions { table: Some(table) },
    ))
}

/// Tauri command: 生成全局枚举 Java 类。
#[tauri::command]
pub async fn generate_enum_command(project: Project, enum_code: String) -> Result<String, String> {
    use aqua_core::generators::java::enum_class::generate_global_enum_class;
    let def = project
        .enums
        .iter()
        .find(|e| e.code == enum_code)
        .ok_or_else(|| format!("枚举不存在: {}", enum_code))?;
    Ok(generate_global_enum_class(&project, def))
}

/// Tauri command: 生成 StrConst 常量类(范围过滤 + 包名/类名)。
#[tauri::command]
pub async fn generate_strconst_command(
    project: Project,
    group: Option<String>,
    package_suffix: Option<String>,
    class_name: Option<String>,
) -> Result<String, String> {
    use aqua_core::generators::strconst::{generate_strconst, StrConstOptions};
    let default = StrConstOptions::default();
    let options = StrConstOptions {
        package_suffix: package_suffix.unwrap_or(default.package_suffix),
        class_name: class_name.unwrap_or(default.class_name),
        group,
    };
    Ok(generate_strconst(&project, &options))
}

/// Tauri command: 生成 ALTER DDL(旧版 vs 当前 project 的 diff)。
#[tauri::command]
pub async fn generate_alter_command(
    old_project: Project,
    new_project: Project,
    dialect: String,
) -> Result<String, String> {
    use aqua_core::alter::{generate_alter, AlterOptions};
    use aqua_core::diff::diff_project;
    let dialect = Dialect::parse(&dialect).ok_or_else(|| format!("不支持的方言: {}", dialect))?;
    let diff = diff_project(&old_project, &new_project);
    Ok(generate_alter(
        &diff,
        &new_project,
        &AlterOptions { dialect },
    ))
}
