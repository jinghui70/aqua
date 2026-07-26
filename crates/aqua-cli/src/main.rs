//! aqua CLI —— aqua-core 之上的薄命令层。
//!
//! 只读 + 生成,无副作用:读表结构(省 token)、生成 entity / DataModel。
//! 所有逻辑复用 aqua-core;本 crate 只做参数解析 + 格式化输出。

mod commands;
mod load;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "aqua",
    about = "aqua 数据表结构工具 —— 读结构 + 生成 entity/DataModel",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 列出所有表组(code + name)
    Groups {
        /// .aqua 文件路径
        file: String,
    },
    /// 列出表(可按组过滤)
    Tables {
        /// 只列该组下的表(表组 code)
        #[arg(long)]
        group: Option<String>,
        /// .aqua 文件路径
        file: String,
    },
    /// 显示单表结构(字段 + 索引)
    Show {
        /// 表 code
        table: String,
        /// .aqua 文件路径
        file: String,
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
        /// .aqua 文件路径
        file: String,
    },
    /// json-ui DataModel JSON
    Datamodel {
        /// 表 code
        table: String,
        /// .aqua 文件路径
        file: String,
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
    match cli.command {
        Command::Groups { file } => commands::query::groups(&file),
        Command::Tables { group, file } => commands::query::tables(&file, group.as_deref()),
        Command::Show { table, file } => commands::query::show(&file, &table),
        Command::Gen { what } => match what {
            GenWhat::Entity { table, file } => commands::gen::entity(&file, &table),
            GenWhat::Datamodel { table, file } => commands::gen::datamodel(&file, &table),
        },
    }
}
