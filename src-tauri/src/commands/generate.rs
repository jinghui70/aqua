//! generate 命令实现。

use std::error::Error;
use std::fs;
use aqua_core::generators::ddl::{generate_ddl, Dialect, DdlOptions};
use aqua_core::generators::java::{generate_java_entity, JavaOptions};
use aqua_core::schema::parse_project;

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

            generate_ddl(&project, &DdlOptions {
                dialect,
                ..Default::default()
            })
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
