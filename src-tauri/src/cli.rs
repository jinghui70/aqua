//! CLI 参数定义。

use clap::{Parser, Subcommand};

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
        #[arg(long = "type")]
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
