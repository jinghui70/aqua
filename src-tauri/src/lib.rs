//! aqua Tauri 壳:GUI + CLI 双模式入口。
//!
//! - 无 args:开 GUI(Tauri webview 加载 app/dist)
//! - 有 args(`aqua generate ...`):走 CLI 模式,调 aqua-core 的 generator,输出 stdout,不开窗
//!
//! commands 是前端(Tauri invoke)与 aqua-core 之间的薄桥。

pub mod cli;
pub mod commands;

use commands::project;

/// 启动 GUI 模式,注册 Tauri commands。
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            project::project_open,
            project::project_save,
            project::project_validate,
        ])
        .run(tauri::generate_context!())
        .expect("aqua 启动失败");
}
