//! aqua CLI -- aqua-core 之上的薄命令层。
//!
//! 只读 + 生成,无副作用:读表结构(省 token)、生成 entity / DataModel。
//! 所有逻辑复用 aqua-core;本 crate 只做参数解析 + 格式化输出。
//! 用法:aqua-cli <file.aqua> <command> [args]

mod commands;
mod load;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "aqua-cli",
    about = "aqua 数据表结构工具 -- 读结构 + 生成 entity/DataModel",
    long_about = None
)]
struct Cli {
    /// .aqua 文件路径
    file: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 列出所有表组(code + name)
    Groups,
    /// 列出表(可按组过滤)
    Tables {
        /// 只列该组下的表(表组 code)
        #[arg(long)]
        group: Option<String>,
    },
    /// 显示单表结构(JSON:字段 + 索引)
    Show {
        /// 表 code
        table: String,
    },
    /// 生成代码产物(输出到 stdout)
    Gen {
        #[command(subcommand)]
        what: GenWhat,
    },
}

#[derive(Subcommand)]
enum GenWhat {
    /// dba 规范 entity Java
    Entity {
        /// 表 code
        table: String,
        /// 自定义包名(不传则不生成 package 声明)
        #[arg(long)]
        package: Option<String>,
        /// 自定义类名(默认 table.code 的 PascalCase)
        #[arg(long = "class-name")]
        class_name: Option<String>,
    },
    /// json-ui DataModel JSON
    Datamodel {
        /// 表 code
        table: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("错误: {e:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let file = &cli.file;
    match cli.command {
        Command::Groups => commands::query::groups(file),
        Command::Tables { group } => commands::query::tables(file, group.as_deref()),
        Command::Show { table } => commands::query::show(file, &table),
        Command::Gen { what } => match what {
            GenWhat::Entity { table, package, class_name } => {
                commands::gen::entity(file, &table, package, class_name)
            }
            GenWhat::Datamodel { table } => commands::gen::datamodel(file, &table),
        },
    }
}
