//! aqua Tauri 壳:GUI + CLI 双模式入口。
//!
//! - 无 args:开 GUI(Tauri webview 加载 app/dist)
//! - 有 args(`aqua generate ...`):走 CLI 模式,调 aqua-core 的 generator,输出 stdout,不开窗
//!
//! commands 是前端(Tauri invoke)与 aqua-core 之间的薄桥。

pub mod cli;
pub mod commands;

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

use commands::{builtin, database, dataset, datasource, generate, import, project};
use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};

/// 构建原生窗口菜单(§6.1),菜单事件通过 "menu" event 发到前端。
fn build_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let file_builder = SubmenuBuilder::new(app, "文件")
        .text("file.new", "新建项目")
        .text("file.open", "打开项目")
        .text("file.recent", "最近项目")
        .text("file.save", "保存")
        .text("file.saveAs", "另存为")
        .text("file.close", "关闭项目");
    // 非 macOS: 文件菜单末尾加退出(macOS 的退出在应用菜单)
    #[cfg(not(target_os = "macos"))]
    let file_builder = file_builder.separator().quit();
    let file = file_builder.build()?;
    let help = SubmenuBuilder::new(app, "帮助")
        .text("help.guide", "用户指南")
        .text("help.about", "关于")
        .build()?;

    let builder = MenuBuilder::new(app);

    // macOS: 第一个 submenu 是应用菜单(显示 app 名),需含 about/quit,
    // 否则业务菜单首项会被当成应用菜单与 app 名重叠。
    #[cfg(target_os = "macos")]
    let builder = {
        let app_menu = SubmenuBuilder::new(app, "aqua")
            .about(None)
            .separator()
            .quit()
            .build()?;
        builder.item(&app_menu)
    };

    builder.items(&[&file, &help]).build()
}

/// 落文件日志(GUI 无 console,Windows 上 spawn connector 的现场靠此定位)。
/// 自实现而非 tauri-plugin-log:确保日志目录创建 + logger 注册可控,避免 plugin setup
/// 失败被吞导致日志静默丢失。Windows 路径: %LOCALAPPDATA%\com.aqua.app\logs\aqua.log
struct FileLogger {
    file: Mutex<File>,
}

impl log::Log for FileLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        if let Ok(mut f) = self.file.lock() {
            let _ = writeln!(f, "{} {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

/// 初始化文件日志:创建日志目录,注册全局 logger(Info 级别)。
fn init_logger<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let dir = match app.path().app_log_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let _ = std::fs::create_dir_all(&dir);
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("aqua.log"))
    {
        Ok(f) => f,
        Err(_) => return,
    };
    let _ = log::set_boxed_logger(Box::new(FileLogger {
        file: Mutex::new(file),
    }));
    // 日志级别由 AQUA_LOG 环境变量控制(info/warn/error/off),默认 info
    let level = std::env::var("AQUA_LOG")
        .ok()
        .and_then(|s| s.parse::<log::LevelFilter>().ok())
        .unwrap_or(log::LevelFilter::Info);
    log::set_max_level(level);
    log::info!("aqua 日志初始化 (级别={}): {}", level, dir.join("aqua.log").display());
}

/// 启动 GUI 模式,注册原生菜单 + Tauri commands。
pub fn run() {
    let confirmed = Arc::new(Mutex::new(false));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .manage(confirmed)
        .menu(build_menu)
        .on_menu_event(|app, event| {
            // 菜单项 id 发到前端,由 useMenuActions 分发
            let _ = app.emit("menu", event.id().0.clone());
        })
        .invoke_handler(tauri::generate_handler![
            project::project_open,
            project::project_save,
            project::project_validate,
            project::update_gitignore,
            generate::generate_ddl_command,
            generate::generate_java_command,
            generate::generate_frontend_json_command,
            generate::generate_strconst_command,
            generate::write_text_file,
            generate::generate_alter_command,
            import::test_connection_command,
            import::import_from_db_command,
            import::list_tables_command,
            dataset::dataset_load,
            dataset::dataset_save,
            dataset::scan_datasets,
            dataset::create_dataset,
            datasource::datasource_load,
            datasource::datasource_save,
            database::list_databases,
            database::install_driver,
            database::uninstall_driver,
            builtin::builtin_biztypes_load,
            set_exit_confirmed,
        ])
        .setup(|app| {
            init_logger(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("aqua 启动失败")
        .run(|app, event| {
            // Command+Q/菜单 quit 触发 ExitRequested;未确认时拦截,emit 前端 confirm dirty
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let c = app.state::<Arc<Mutex<bool>>>();
                let confirmed = c.lock().unwrap();
                if !*confirmed {
                    api.prevent_exit();
                    drop(confirmed);
                    let _ = app.emit("confirm-exit", ());
                }
            }
        });
}

#[tauri::command]
fn set_exit_confirmed(confirmed: tauri::State<Arc<Mutex<bool>>>) {
    *confirmed.lock().unwrap() = true;
}

#[cfg(test)]
mod tests {
    /// 确保启用了 log 的 max_level_info feature:否则 log::info! 在编译期被静态过滤
    /// (STATIC_MAX_LEVEL < Info),运行时 set_max_level 无效,诊断日志不落盘。
    /// (Windows connector 问题曾因日志静默失效无法定位。)
    #[test]
    fn log_info_level_not_filtered_out() {
        assert!(
            log::STATIC_MAX_LEVEL >= log::LevelFilter::Info,
            "log crate 未启用 max_level_info/release_max_level_info feature,info 日志会被静态过滤"
        );
    }
}
