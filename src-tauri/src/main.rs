// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod commands;

use clap::Parser;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // CLI 模式
        let cli = cli::Cli::parse();
        match cli.command {
            cli::Commands::Generate { type_, input, dialect, table, output } => {
                if let Err(e) = commands::generate::handle_generate(type_, input, dialect, table, output) {
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
