use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, Submenu, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, State,
};

// 引入 Tauri V2 的 shell 扩展包
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

// 1. 结构体：用于在全局状态中安全地保存和管理 Python 子进程
struct PythonServiceState {
    child: Mutex<Option<CommandChild>>,
}

// 封装一个启动 Python 服务的函数
// 修改后的启动函数
fn start_python_sidecar(app_handle: &tauri::AppHandle, port: &str, path: &str) -> Option<CommandChild> {
    println!("正在通过 Sidecar 启动 Python 编译服务，端口: {}, 目录: {}", port, path);

    // 1. 使用 shell().sidecar() 代替普通的 command()
    let shell = app_handle.shell();
    let cmd = shell.sidecar("pyserver")
        .expect("未能找到声明的 pyserver sidecar 程序")
        .args([port, "-d", path]);
        // .args([port, path]);

    // 2. 产生子进程
    match cmd.spawn() {
        Ok((_rx, child)) => {
            println!("🟢 PyInstaller Sidecar 服务启动成功！");
            Some(child)
        }
        Err(e) => {
            eprintln!("🔴 无法启动 Sidecar 服务: {}", e);
            None
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 2. 注册 shell 插件
        .plugin(tauri_plugin_shell::init())
        // 3. 管理全局状态
        .manage(PythonServiceState {
            child: Mutex::new(None),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 默认配置（后续你可以用 tauri-plugin-store 改为读取本地持久化数据）
            let default_port = "8080".to_string();
            // 获取系统的 Downloads 目录路径 (返回的是一个 std::path::PathBuf)
            let download_path_buf = app.path().download_dir().expect("无法获取下载目录");
            let default_path = download_path_buf.to_string_lossy().into_owned();

            // 克隆一份给初始启动使用（因为下面闭包还要用）
            let path_for_init = default_path.clone();
            let port_for_init = default_port.clone();

            // 4. 启动初始化服务并存入状态机
            //   调用新的 Sidecar 启动函数
            let child_proc = start_python_sidecar(&app_handle, &port_for_init, &path_for_init);
            if let Some(c) = child_proc {
                let state = app.state::<PythonServiceState>();
                *state.child.lock().unwrap() = Some(c);
            }

            // ---- 构建系统托盘菜单 ----
            let quit_i = MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>)?;
            let restart_i = MenuItem::with_id(app, "restart", "重启 Python 服务", true, None::<&str>)?;
            let status_i = MenuItem::with_id(app, "status", format!("🟢 服务运行在端口: {}", default_port), false, None::<&str>)?;

            let tray_menu = Menu::with_items(
                app,
                &[
                    &status_i,
                    &PredefinedMenuItem::separator(app)?,
                    &restart_i,
                    &PredefinedMenuItem::separator(app)?,
                    &quit_i,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(move |app_handle, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            // 5. 优雅退出：手动干掉子进程（虽然随主进程退出也会释放，但主动关闭更安全）
                            let state = app_handle.state::<PythonServiceState>();
                            if let Some(child) = state.child.lock().unwrap().take() {
                                let _ = child.kill();
                                println!("已杀死 Python 服务进程");
                            }
                            app_handle.exit(0);
                        }
                        "restart" => {
                            let state = app_handle.state::<PythonServiceState>();
                            // 先干掉老进程
                            if let Some(child) = state.child.lock().unwrap().take() {
                                let _ = child.kill();
                                println!("旧服务已清理");
                            }
                            // 重新拉起新进程
                            let new_child = start_python_sidecar(app_handle, &default_port, &default_path);
                            if let Some(c) = new_child {
                                *state.child.lock().unwrap() = Some(c);
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}