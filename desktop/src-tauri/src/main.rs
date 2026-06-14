//! Desktop shell: a Tauri 2 window around the server binary, bundled as a
//! sidecar. The sidecar is spawned on launch — in both dev and release — and
//! killed on exit, so `just desktop-dev` and a packaged build both work with a
//! single command, without starting the backend separately. The sidecar binary
//! is produced by `scripts/bundle-sidecar.sh`, which `bun run dev`/`build` run
//! automatically before invoking Tauri.
//!
//! If you prefer to iterate on the backend with a hot-reloading `cargo run`,
//! the spawned sidecar simply fails to bind the port and exits while your
//! `cargo run` serves requests — the window keeps working either way.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_shell::ShellExt;

struct SidecarChild(std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let (_events, child) = app
                .shell()
                .sidecar("app-starter")
                .expect("sidecar missing: run scripts/bundle-sidecar.sh first")
                .current_dir(&data_dir)
                .env("DATABASE_URL", "sqlite://app.db?mode=rwc")
                .spawn()
                .expect("failed to spawn server sidecar");
            app.manage(SidecarChild(std::sync::Mutex::new(Some(child))));
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
            if let Some(state) = app_handle.try_state::<SidecarChild>() {
                if let Some(child) = state.0.lock().unwrap().take() {
                    let _ = child.kill();
                }
            }
        }
        _ => {}
    });
}
