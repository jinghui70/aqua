//! aqua Tauri 壳:GUI + CLI 双模式入口。
//!
//! - 无 args:开 GUI(Tauri webview 加载 app/dist)
//! - 有 args(`aqua generate ...`):走 CLI 模式,调 aqua-core 的 generator,输出 stdout,不开窗
//!
//! commands 是前端(Tauri invoke)与 aqua-core 之间的薄桥。

use tauri::Manager;

/// 示例 command: 验证前后端链路通。后续替换为真实能力(project open/save, generate, import...)。
#[tauri::command]
fn greet(name: &str) -> String {
    format!("aqua v2 (Rust+Tauri) 你好, {}", name)
}

pub fn run() {
    // TODO(CLI): 解析 std::env::args(),若有 `generate` 子命令则走 CLI 模式(调 aqua_core::generators),
    //            输出 stdout 后 exit,不启动 webview。见 docs/architecture.md §6。
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("aqua 启动失败");
}
