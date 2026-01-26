mod cli;
mod window_customizer;
mod config_helpers;
mod config_api;
mod updater;

use cli::{get_embedded_cli_path, install_cli, sync_cli};
use config_helpers::backup_and_reset_config;
use config_api::{read_opencode_config, write_opencode_config};
use std::fs::read_to_string;
use dirs::home_dir;
use futures::FutureExt;
use std::{
    collections::VecDeque,
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{
    path::BaseDirectory, AppHandle, LogicalSize, Manager, RunEvent, State, WebviewUrl,
    WebviewWindow,
};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::net::TcpSocket;

use crate::window_customizer::PinchZoomDisablePlugin;
use std::fs as stdfs;
use std::time::SystemTime;
use chrono::Utc;
use std::path::PathBuf;

#[derive(Clone)]
struct ServerState {
    child: Arc<Mutex<Option<CommandChild>>>,
    status: futures::future::Shared<tokio::sync::oneshot::Receiver<Result<(), String>>>,
}

impl ServerState {
    pub fn new(
        child: Option<CommandChild>,
        status: tokio::sync::oneshot::Receiver<Result<(), String>>,
    ) -> Self {
        Self {
            child: Arc::new(Mutex::new(child)),
            status: status.shared(),
        }
    }

    pub fn set_child(&self, child: Option<CommandChild>) {
        *self.child.lock().unwrap() = child;
    }
}

#[derive(Clone)]
struct LogState(Arc<Mutex<VecDeque<String>>>);

const MAX_LOG_ENTRIES: usize = 200;

#[tauri::command]
fn kill_sidecar(app: AppHandle) {
    let Some(server_state) = app.try_state::<ServerState>() else {
        println!("Server not running");
        return;
    };

    let Some(server_state) = server_state
        .child
        .lock()
        .expect("Failed to acquire mutex lock")
        .take()
    else {
        println!("Server state missing");
        return;
    };

    let _ = server_state.kill();

    println!("Killed server");
}

async fn get_logs(app: AppHandle) -> Result<String, String> {
    let log_state = app.try_state::<LogState>().ok_or("Log state not found")?;

    let logs = log_state
        .0
        .lock()
        .map_err(|_| "Failed to acquire log lock")?;

    Ok(logs.iter().cloned().collect::<Vec<_>>().join(""))
}

#[tauri::command]
async fn ensure_server_started(state: State<'_, ServerState>) -> Result<(), String> {
    state
        .status
        .clone()
        .await
        .map_err(|_| "Failed to get server status".to_string())?
}

fn get_sidecar_port() -> u32 {
    // Prefer OPENCODE_PORT env if set
    if let Some(port_str) = option_env!("OPENCODE_PORT") {
        if let Ok(port) = port_str.parse::<u32>() {
            return port;
        }
    }
    if let Ok(port_str) = std::env::var("OPENCODE_PORT") {
        if let Ok(port) = port_str.parse::<u32>() {
            return port;
        }
    }

    // Otherwise scan for an available port in a high unprivileged range (e.g. 10420..65535)
    for port in 10420..=65535u32 {
        let addr = format!("127.0.0.1:{}", port);
        if std::net::TcpListener::bind(&addr).is_ok() {
            return port;
        }
    }

    // Fallback to ephemeral port
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to find free port")
        .local_addr()
        .expect("Failed to get local address")
        .port() as u32
}

#[cfg(not(target_os = "windows"))]
fn get_user_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn spawn_sidecar(app: &AppHandle, port: u32) -> CommandChild {
    // Ensure required user config directories exist and create defaults if necessary
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cci_config_dir = std::path::PathBuf::from(&home).join("cci-config").join("opencode");
    let cci_pref_dir = std::path::PathBuf::from(&home).join("cci-tech").join("open-persona-v3");
    let agents_dir = cci_pref_dir.join("agents");
    let backups_dir = cci_pref_dir.join("backups");

    if let Err(e) = stdfs::create_dir_all(&cci_config_dir) {
        eprintln!("Failed to create config dir {:?}: {}", cci_config_dir, e);
    }
    if let Err(e) = stdfs::create_dir_all(&agents_dir) {
        eprintln!("Failed to create agents dir {:?}: {}", agents_dir, e);
    }
    if let Err(e) = stdfs::create_dir_all(&backups_dir) {
        eprintln!("Failed to create backups dir {:?}: {}", backups_dir, e);
    }

    // Ensure opencode.jsonc exists in cci_config_dir
    let opencode_cfg = cci_config_dir.join("opencode.jsonc");
    if !opencode_cfg.exists() {
        // create a minimal default config
        let default = r#"{
  "$schema": "https://opencode.ai/config.json",
  "model": "",
  "small_model": "",
  "provider": {}
}"#;
        if let Err(e) = stdfs::write(&opencode_cfg, default) {
            eprintln!("Failed to write default config {:?}: {}", opencode_cfg, e);
        }
    }

    let log_state = app.state::<LogState>();
    let log_state_clone = log_state.inner().clone();

    let state_dir = app
        .path()
        .resolve("", BaseDirectory::AppLocalData)
        .expect("Failed to resolve app local data dir");

    #[cfg(target_os = "windows")]
    let (mut rx, child) = app
        .shell()
        .sidecar("opencode-cli")
        .unwrap()
        .env("OPENCODE_EXPERIMENTAL_ICON_DISCOVERY", "true")
        .env("OPENCODE_CLIENT", "desktop")
        .env("XDG_STATE_HOME", &state_dir)
        // pass our override config path and config dir to the sidecar
        .env("OPENCODE_CONFIG", &opencode_cfg)
        .env("OPENCODE_CONFIG_DIR", &cci_pref_dir)
        .args(["serve", &format!("--port={port}")])
        .spawn()
        .expect("Failed to spawn opencode");

    #[cfg(not(target_os = "windows"))]
    let (mut rx, child) = {
        let sidecar = get_sidecar_path();
        let shell = get_user_shell();
        app.shell()
            .command(&shell)
            .env("OPENCODE_EXPERIMENTAL_ICON_DISCOVERY", "true")
            .env("OPENCODE_CLIENT", "desktop")
            .env("XDG_STATE_HOME", &state_dir)
            // pass our override config path and config dir to the sidecar
            .env("OPENCODE_CONFIG", &opencode_cfg)
            .env("OPENCODE_CONFIG_DIR", &cci_pref_dir)
            .args([
                "-il",
                "-c",
                &format!("\"{}\" serve --port={}", sidecar.display(), port),
            ])
            .spawn()
            .expect("Failed to spawn opencode")
    };
    let log_state = app.state::<LogState>();
    let log_state_clone = log_state.inner().clone();

    let state_dir = app
        .path()
        .resolve("", BaseDirectory::AppLocalData)
        .expect("Failed to resolve app local data dir");

    #[cfg(target_os = "windows")]
    let (mut rx, child) = app
        .shell()
        .sidecar("opencode-cli")
        .unwrap()
        .env("OPENCODE_EXPERIMENTAL_ICON_DISCOVERY", "true")
        .env("OPENCODE_CLIENT", "desktop")
        .env("XDG_STATE_HOME", &state_dir)
        .args(["serve", &format!("--port={port}")])
        .spawn()
        .expect("Failed to spawn opencode");

    #[cfg(not(target_os = "windows"))]
    let (mut rx, child) = {
        let sidecar = get_sidecar_path();
        let shell = get_user_shell();
        app.shell()
            .command(&shell)
            .env("OPENCODE_EXPERIMENTAL_ICON_DISCOVERY", "true")
            .env("OPENCODE_CLIENT", "desktop")
            .env("XDG_STATE_HOME", &state_dir)
            .args([
                "-il",
                "-c",
                &format!("\"{}\" serve --port={}", sidecar.display(), port),
            ])
            .spawn()
            .expect("Failed to spawn opencode")
    };

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    print!("{line}");

                    // Store log in shared state
                    if let Ok(mut logs) = log_state_clone.0.lock() {
                        logs.push_back(format!("[STDOUT] {}", line));
                        // Keep only the last MAX_LOG_ENTRIES
                        while logs.len() > MAX_LOG_ENTRIES {
                            logs.pop_front();
                        }
                    }
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    eprint!("{line}");

                    // Store log in shared state
                    if let Ok(mut logs) = log_state_clone.0.lock() {
                        logs.push_back(format!("[STDERR] {}", line));
                        // Keep only the last MAX_LOG_ENTRIES
                        while logs.len() > MAX_LOG_ENTRIES {
                            logs.pop_front();
                        }
                    }
                }
                _ => {}
            }
        }
    });

    child
}

async fn is_server_running(port: u32) -> bool {
    TcpSocket::new_v4()
        .unwrap()
        .connect(SocketAddr::new(
            "127.0.0.1".parse().expect("Failed to parse IP"),
            port as u16,
        ))
        .await
        .is_ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let updater_enabled = option_env!("TAURI_SIGNING_PRIVATE_KEY").is_some();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus existing window when another instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_pty::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(PinchZoomDisablePlugin)
        .invoke_handler(tauri::generate_handler![
            kill_sidecar,
            install_cli,
            ensure_server_started,
            get_embedded_cli_path,
            backup_and_reset_config,
            // config read/write
            read_opencode_config,
            write_opencode_config,
            updater_check_and_install
        ])
        .setup(move |app| {
            let app = app.handle().clone();

            // Initialize log state
            app.manage(LogState(Arc::new(Mutex::new(VecDeque::new()))));

            // Get port and create window immediately for faster perceived startup
            let port = get_sidecar_port();

            let primary_monitor = app.primary_monitor().ok().flatten();
            let size = primary_monitor
                .map(|m| m.size().to_logical(m.scale_factor()))
                .unwrap_or(LogicalSize::new(1920, 1080));

            // Create window immediately with serverReady = false
            let window_builder =
                WebviewWindow::builder(&app, "main", WebviewUrl::App("/".into()))
                    .title("OpenCode")
                    .inner_size(size.width as f64, size.height as f64)
                    .decorations(true)
                    .zoom_hotkeys_enabled(true)
                    .disable_drag_drop_handler()
                    .initialization_script(format!(
                        r#"
                      window.__OPENCODE__ ??= {{}};
                      window.__OPENCODE__.updaterEnabled = {updater_enabled};
                      window.__OPENCODE__.port = {port};
                    "#
                    ));

            #[cfg(target_os = "macos")]
            let window_builder = window_builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);

            let window = window_builder.build().expect("Failed to create window");

            let (tx, rx) = tokio::sync::oneshot::channel();
            app.manage(ServerState::new(None, rx));

            {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let should_spawn_sidecar = !is_server_running(port).await;

                    let (child_opt, res) = if should_spawn_sidecar {
                        // Attempt to start the sidecar on multiple ports in case of port conflicts.
                        let mut attempt: usize = 0;
                        let mut final_res: Result<(), String> = Err("no attempt made".to_string());
                        let mut chosen_child: Option<CommandChild> = None;

                        while attempt < 6 {
                            let try_port = get_sidecar_port();
                            println!("Attempting to spawn sidecar on port {} (attempt {})", try_port, attempt + 1);

                            let child = spawn_sidecar(&app, try_port);

                            // wait briefly for the server to come up on this port
                            let start = Instant::now();
                            let mut ok = false;
                            while start.elapsed() < Duration::from_secs(3) {
                                if is_server_running(try_port).await {
                                    ok = true;
                                    break;
                                }
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }

                            if ok {
                                final_res = Ok(());
                                chosen_child = Some(child);
                                println!("Sidecar started successfully on port {}", try_port);
                                break;
                            }

                            // Not up yet. Inspect logs to see if it failed due to port bind or similar.
                            let logs = get_logs(app.clone()).await.unwrap_or_default();
                            if logs.contains("Failed to start server on port") || logs.contains("address already in use") {
                                // kill the child and try next port
                                let _ = child.kill();
                                println!("Port {} appears in use or binding failed; retrying", try_port);
                                attempt += 1;
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            // Give a bit more time (extended wait) before deciding to retry
                            let extra_deadline = Instant::now() + Duration::from_secs(4);
                            while Instant::now() < extra_deadline {
                                if is_server_running(try_port).await {
                                    ok = true;
                                    break;
                                }
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }

                            if ok {
                                final_res = Ok(());
                                chosen_child = Some(child);
                                break;
                            }

                            // Still not started; kill and retry
                            let _ = child.kill();
                            attempt += 1;
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }

                        (chosen_child, final_res)
                    } else {
                        (None, Ok(()))
                    };

                    app.state::<ServerState>().set_child(child_opt);

                    if res.is_ok() {
                        let _ = window.eval("window.__OPENCODE__.serverReady = true;");
                    }

                    let _ = tx.send(res);
                });
            }

            {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = sync_cli(app) {
                        eprintln!("Failed to sync CLI: {e}");
                    }
                });
            }

            Ok(())
        });

    if updater_enabled {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                println!("Received Exit");

                kill_sidecar(app.clone());
            }
        });
}
