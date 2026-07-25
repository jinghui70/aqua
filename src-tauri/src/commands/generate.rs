//! generate 命令实现(Tauri commands)。

use aqua_core::generators::ddl::{generate_ddl, DdlOptions, Dialect};
use aqua_core::generators::java::{generate_java_entity, JavaOptions};
use aqua_core::schema::Project;

/// Tauri command: 生成 DDL。
#[tauri::command]
pub async fn generate_ddl_command(
    project: Project,
    dialect: String,
    tables: Option<Vec<String>>,
    group: Option<String>,
    drop_if_exist: Option<bool>,
    dataset_path: Option<String>,
) -> Result<String, String> {
    let dialect = Dialect::parse(&dialect).ok_or_else(|| format!("不支持的方言: {}", dialect))?;
    let options = DdlOptions {
        dialect,
        tables,
        group,
        drop_if_exist: drop_if_exist.unwrap_or(true),
    };
    let mut ddl = generate_ddl(&project, &options);
    // 选中数据集时,追加该数据集的 INSERT 语句(按同样表过滤)
    if let Some(path) = dataset_path {
        let (entries, _) = aqua_core::dataset::load_dataset(&path, &project)
            .map_err(|e| e.to_string())?;
        let insert = aqua_core::generators::ddl::generate_insert(&project, &entries, &options);
        if !insert.is_empty() {
            if !ddl.is_empty() {
                ddl.push('\n');
            }
            ddl.push_str(&insert);
        }
    }
    Ok(ddl)
}

/// Tauri command: 生成 Java 实体类(支持配置: 包名/类名/Lombok,注释始终生成)。
#[tauri::command]
pub async fn generate_java_command(
    project: Project,
    table: String,
    use_lombok: Option<bool>,
    package: Option<String>,
    class_name: Option<String>,
) -> Result<String, String> {
    let options = JavaOptions {
        use_lombok: use_lombok.unwrap_or(true),
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

/// Tauri command: 生成 StrConst 常量类(全部表或按分组;类名固定 DatabaseConstants)。
#[tauri::command]
pub async fn generate_strconst_command(
    project: Project,
    group: Option<String>,
) -> Result<String, String> {
    use aqua_core::generators::strconst::{generate_strconst, StrConstOptions};
    let options = StrConstOptions { group };
    Ok(generate_strconst(&project, &options))
}

/// Tauri command: 写文本文件(导出保存用)。
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, &content).await.map_err(|e| format!("写文件失败: {}", e))
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
