//! Desktop shell: a Tauri 2 window around the server binary, bundled as a
//! sidecar. Release builds spawn the sidecar and kill it on exit. In dev,
//! run the backend yourself (`cargo run` at the repo root) and Tauri loads
//! the Vite dev server.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(debug_assertions))]
struct SidecarChild(std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            #[cfg(not(debug_assertions))]
            {
                use tauri::Manager;
                use tauri_plugin_shell::ShellExt;

                let data_dir = _app.path().app_data_dir()?;
                std::fs::create_dir_all(&data_dir)?;

                let (_events, child) = _app
                    .shell()
                    .sidecar("app-starter")
                    .expect("sidecar missing: run scripts/bundle-sidecar.sh first")
                    .current_dir(&data_dir)
                    .env("DATABASE_URL", "sqlite://app.db?mode=rwc")
                    .spawn()
                    .expect("failed to spawn server sidecar");
                _app.manage(SidecarChild(std::sync::Mutex::new(Some(child))));
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::WindowEvent {
            label,
            event: tauri::WindowEvent::CloseRequested { .. },
            ..
        } if label == "main" => app_handle.exit(0),
        tauri::RunEvent::Exit => {
            #[cfg(not(debug_assertions))]
            {
                use tauri::Manager;
                if let Some(state) = app_handle.try_state::<SidecarChild>() {
                    if let Some(child) = state.0.lock().unwrap().take() {
                        let _ = child.kill();
                    }
                }
            }
        }
        _ => {}
    });
}
