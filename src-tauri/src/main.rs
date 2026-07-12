// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // CLI 模式
        let cli = aqua::cli::Cli::parse();
        match cli.command {
            aqua::cli::Commands::Generate {
                type_,
                input,
                dialect,
                table,
                output,
            } => {
                if let Err(e) =
                    aqua::commands::generate::handle_generate(type_, input, dialect, table, output)
                {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // GUI 模式
        aqua::run();
    }
}
