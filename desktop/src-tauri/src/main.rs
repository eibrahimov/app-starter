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
//!
//! Cheap-safety defaults (issue #50, see docs/desktop-features.md §3):
//! - a single-instance guard so a second launch re-focuses the existing window
//!   instead of spawning a second axum sidecar that races `app.db` and the port;
//! - window-state persistence so the window reopens where the user left it;
//! - the sidecar's stdout/stderr/exit are drained into `tauri-plugin-log` so a
//!   silent backend boot failure leaves a trace instead of just a broken UI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

struct SidecarChild(std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>);

fn main() {
    let mut builder = tauri::Builder::default();

    // The single-instance guard MUST be the first plugin registered: it short-
    // circuits a second launch before any of `setup` runs, so a duplicate
    // process never spawns a second sidecar contending for `sqlite://app.db`
    // (mode=rwc) and port 8080. Desktop-only — both this plugin and window-state
    // are unavailable on mobile, and the sidecar model does not run there anyway.
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                // Re-surface the existing window for the second launch. tauri#12936:
                // `set_focus` is a no-op on a hidden or minimized window, so show
                // and unminimize first.
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_window_state::Builder::default().build());
    }

    let app = builder
        .plugin(tauri_plugin_shell::init())
        // Unified on-device log sink: stdout (visible under `just desktop-dev`)
        // plus a rotating file in the OS log directory (the durable trace for a
        // packaged build). The sidecar drain below writes through this.
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app-starter-desktop".into()),
                    }),
                ])
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            // The sidecar port and the webview's API base URL are the two ends of
            // one contract (docs/api-endpoint.md). Pass PORT explicitly so the
            // shell no longer silently relies on the server binary's own default;
            // a desktop launch can override it, in which case the webview must be
            // built with a matching VITE_API_BASE_URL. Default is unchanged (8080).
            let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned());

            let (mut events, child) = app
                .shell()
                .sidecar("app-starter")
                .expect("sidecar missing: run scripts/bundle-sidecar.sh first")
                .current_dir(&data_dir)
                .env("DATABASE_URL", "sqlite://app.db?mode=rwc")
                .env("PORT", &port)
                .spawn()
                .expect("failed to spawn server sidecar");

            // Drain the sidecar's event stream. Previously this receiver was
            // dropped (`let (_events, child) = …`), so a failed bind, a stale-
            // migration panic, or any non-zero exit vanished with no trace and
            // left a silently broken UI. Forwarded lines keep the backend's own
            // formatting; the log plugin adds the shell-side timestamp/level (a
            // deliberate, minor double-stamp on already-structured backend lines).
            tauri::async_runtime::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        CommandEvent::Stdout(bytes) => {
                            log::info!(target: "sidecar", "{}", String::from_utf8_lossy(&bytes).trim_end());
                        }
                        CommandEvent::Stderr(bytes) => {
                            log::warn!(target: "sidecar", "{}", String::from_utf8_lossy(&bytes).trim_end());
                        }
                        CommandEvent::Terminated(payload) => {
                            log::error!(
                                target: "sidecar",
                                "server sidecar exited (code={:?}, signal={:?})",
                                payload.code,
                                payload.signal
                            );
                        }
                        CommandEvent::Error(err) => {
                            log::error!(target: "sidecar", "server sidecar stream error: {err}");
                        }
                        _ => {}
                    }
                }
            });

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
