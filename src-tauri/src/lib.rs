// 1. 修改顶部的 use 引入，添加 PredefinedMenuItem
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu}, // 👈 这里加上 PredefinedMenuItem
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let restart_i = MenuItem::with_id(app, "restart", "重启服务", true, None::<&str>)?;
            let status_i =
                MenuItem::with_id(app, "status", "🟢 Python 服务运行中", false, None::<&str>)?;

            let help_menu = Submenu::with_items(
                app,
                "帮助与反馈",
                true,
                &[
                    &MenuItem::with_id(app, "docs", "查看文档", true, None::<&str>)?,
                    &MenuItem::with_id(app, "about", "关于", true, None::<&str>)?,
                ],
            )?;

            // 2. 将组合主菜单这里的 MenuItem::separator 改为 PredefinedMenuItem::separator
            let tray_menu = Menu::with_items(
                app,
                &[
                    &status_i,
                    &PredefinedMenuItem::separator(app)?, // 👈 改为 PredefinedMenuItem
                    &restart_i,
                    &help_menu,
                    &PredefinedMenuItem::separator(app)?, // 👈 改为 PredefinedMenuItem
                    &quit_i,
                ],
            )?;

            // 后面保持不变 ...
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "restart" => println!("触发重启 Python 服务..."),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
