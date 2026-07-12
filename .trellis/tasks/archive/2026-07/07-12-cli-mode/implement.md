# cli-mode 实现计划

## 实施步骤

1. [ ] 添加 clap 依赖到 src-tauri/Cargo.toml
2. [ ] `src-tauri/src/cli.rs` - CLI 参数定义
3. [ ] `src-tauri/src/commands/mod.rs` - commands 模块
4. [ ] `src-tauri/src/commands/generate.rs` - generate 命令实现
5. [ ] 更新 `src-tauri/src/main.rs` - 入口判断
6. [ ] 手动测试

## CLI 参数设计

```rust
#[derive(Parser, Debug)]
#[command(name = "aqua")]
#[command(about = "数据库结构管理工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 生成 DDL/Java/JSON/StrConst
    Generate {
        /// 生成类型: ddl | java | json | strconst
        #[arg(short, long)]
        type_: String,

        /// 输入 schema.json 路径
        #[arg(short, long)]
        input: String,

        /// DDL 方言: mysql | postgresql | oracle | ...
        #[arg(short, long)]
        dialect: Option<String>,

        /// Java/DDL 表名(Java 必需)
        #[arg(short, long)]
        table: Option<String>,

        /// 输出路径(为空则 stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
}
```

## generate 命令实现

```rust
pub fn handle_generate(args: GenerateArgs) -> Result<(), Box<dyn Error>> {
    // 1. 读取 input JSON
    let json_str = fs::read_to_string(&args.input)?;
    let value: serde_json::Value = serde_json::from_str(&json_str)?;
    let project = aqua_core::schema::parse_project(value)?;

    // 2. 根据 type 调用 generator
    let output = match args.type_.as_str() {
        "ddl" => {
            let dialect = Dialect::parse(&args.dialect.unwrap_or("mysql".to_string()))
                .ok_or("Invalid dialect")?;
            aqua_core::generators::ddl::generate_ddl(&project, &DdlOptions {
                dialect,
                ..Default::default()
            })
        }
        "java" => {
            let table = args.table.ok_or("--table required for java")?;
            aqua_core::generators::java::generate_java_entity(&project, &table, &JavaOptions::default())?
        }
        _ => return Err("Unsupported type".into()),
    };

    // 3. 输出
    if let Some(out_path) = args.output {
        fs::write(out_path, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
```

## main.rs 入口

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // CLI 模式
        let cli = Cli::parse();
        match cli.command {
            Commands::Generate(args) => {
                if let Err(e) = handle_generate(args) {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // GUI 模式
        tauri::Builder::default()
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
```

## 测试命令

```bash
# DDL 生成
cargo run -p aqua -- generate --type ddl --input crates/aqua-core/tests/fixtures/valid-full.json --dialect mysql

# Java 生成
cargo run -p aqua -- generate --type java --input crates/aqua-core/tests/fixtures/valid-full.json --table SYS_USER

# 输出到文件
cargo run -p aqua -- generate --type ddl --input crates/aqua-core/tests/fixtures/valid-full.json --output /tmp/schema.sql
```
