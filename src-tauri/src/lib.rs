//! aqua Tauri 壳:GUI + CLI 双模式入口。
//!
//! - 无 args:开 GUI(Tauri webview 加载 app/dist)
//! - 有 args(`aqua generate ...`):走 CLI 模式,调 aqua-core 的 generator,输出 stdout,不开窗
//!
//! commands 是前端(Tauri invoke)与 aqua-core 之间的薄桥。

pub mod cli;
pub mod commands;

use commands::{generate, import, project};
use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::Emitter;

/// 构建原生窗口菜单(§6.1),菜单事件通过 "menu" event 发到前端。
fn build_menu<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let file = SubmenuBuilder::new(app, "文件")
        .text("file.new", "新建项目")
        .text("file.open", "打开项目")
        .text("file.save", "保存")
        .text("file.saveAs", "另存为")
        .build()?;
    let config = SubmenuBuilder::new(app, "配置")
        .text("config.biztype", "业务类型管理")
        .text("config.enum", "枚举管理")
        .text("config.dataset", "数据集管理")
        .text("config.datasource", "数据源配置")
        .build()?;
    let export = SubmenuBuilder::new(app, "导出")
        .text("export.ddl", "DDL")
        .text("export.diff", "diff")
        .text("export.strconst", "StrConst")
        .build()?;
    let help = SubmenuBuilder::new(app, "帮助")
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

    builder.items(&[&file, &config, &export, &help]).build()
}

/// 启动 GUI 模式,注册原生菜单 + Tauri commands。
pub fn run() {
    tauri::Builder::default()
        .menu(|handle| build_menu(handle))
        .on_menu_event(|app, event| {
            // 菜单项 id 发到前端,由 useMenuActions 分发
            let _ = app.emit("menu", event.id().0.clone());
        })
        .invoke_handler(tauri::generate_handler![
            project::project_open,
            project::project_save,
            project::project_validate,
            generate::generate_ddl_command,
            generate::generate_java_command,
            generate::generate_frontend_json_command,
            generate::generate_enum_command,
            generate::generate_strconst_command,
            generate::generate_alter_command,
            import::test_connection_command,
            import::import_from_db_command,
        ])
        .run(tauri::generate_context!())
        .expect("aqua 启动失败");
}
