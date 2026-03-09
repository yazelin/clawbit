mod emotion;
mod hooks;
mod ipc;
mod models;
mod session;
mod settings;
mod sound;
mod state;
mod stats;

use emotion::{EmotionAnalyzer, EmotionState};
use models::{HookEvent, HookEventType};
use session::SessionStore;
use settings::AppSettings;
use stats::StatsCollector;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use tokio::sync::mpsc;

struct AppState {
    sessions: Mutex<SessionStore>,
    stats: Mutex<StatsCollector>,
    settings: Mutex<AppSettings>,
    emotions: Mutex<HashMap<String, EmotionState>>,
}

#[tauri::command]
fn get_sessions(state: State<AppState>) -> Vec<serde_json::Value> {
    let sessions = state.sessions.lock().unwrap();
    sessions
        .get_sessions()
        .iter()
        .map(|s| serde_json::to_value(s).unwrap())
        .collect()
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> serde_json::Value {
    let stats = state.stats.lock().unwrap();
    serde_json::to_value(stats.get_aggregate()).unwrap()
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> serde_json::Value {
    let settings = state.settings.lock().unwrap();
    serde_json::to_value(&*settings).unwrap()
}

#[tauri::command]
fn update_settings(state: State<AppState>, new_settings: AppSettings) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    *settings = new_settings;
    settings.save()
}

#[tauri::command]
fn install_hooks() -> Result<String, String> {
    hooks::installer::install()?;
    Ok("Hooks installed".into())
}

#[tauri::command]
fn uninstall_hooks() -> Result<String, String> {
    hooks::installer::uninstall()?;
    Ok("Hooks removed".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<HookEvent>();

    let app_state = AppState {
        sessions: Mutex::new(SessionStore::new()),
        stats: Mutex::new(StatsCollector::new()),
        settings: Mutex::new(AppSettings::load()),
        emotions: Mutex::new(HashMap::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            // Set webview background to transparent (needed on Linux)
            if let Some(window) = app.get_webview_window("main") {
                use tauri::window::Color;
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
            }

            let handle = app.handle().clone();

            // Start IPC server
            let ipc_server = ipc::IpcServer::new(event_tx);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ipc_server.start().await {
                    eprintln!("IPC server error: {}", e);
                }
            });

            // Event processing loop
            let handle2 = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    let state = handle2.state::<AppState>();

                    // Update session
                    {
                        let mut sessions = state.sessions.lock().unwrap();
                        sessions.handle_event(&event);
                    }

                    // Update stats
                    {
                        let mut stats = state.stats.lock().unwrap();
                        if event.event == HookEventType::UserPromptSubmit {
                            stats.record_turn(&event.session_id);
                        }
                        if let Some(ref tool) = event.tool {
                            stats.record_tool_call(&event.session_id, tool);
                        }
                    }

                    // Analyze emotion for user prompts
                    if let Some(ref prompt) = event.user_prompt {
                        let prompt = prompt.clone();
                        let session_id = event.session_id.clone();
                        let handle3 = handle2.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            if let Some(result) = EmotionAnalyzer::analyze(&prompt) {
                                let state = handle3.state::<AppState>();
                                let mut emotions = state.emotions.lock().unwrap();
                                let emotion_state = emotions
                                    .entry(session_id.clone())
                                    .or_insert_with(EmotionState::new);
                                emotion_state.apply(&result.emotion, result.intensity);
                                let emotion = emotion_state.current_emotion();
                                drop(emotions);
                                let mut sessions = state.sessions.lock().unwrap();
                                sessions.update_emotion(&session_id, emotion);
                            }
                        });
                    }

                    // Emit update to frontend
                    let _ = handle2.emit("pet-update", ());
                }
            });

            // Periodic tasks: decay emotions, check sleepers
            let handle4 = handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let state = handle4.state::<AppState>();
                    {
                        let mut emotions = state.emotions.lock().unwrap();
                        for es in emotions.values_mut() {
                            es.decay();
                        }
                    }
                    {
                        let mut sessions = state.sessions.lock().unwrap();
                        sessions.check_sleepers();
                        sessions.remove_ended();
                    }
                    let _ = handle4.emit("pet-update", ());
                }
            });

            // Setup system tray
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let show = MenuItem::with_id(app, "show", "Show Clawbit", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app: &tauri::AppHandle, event: tauri::menu::MenuEvent| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_stats,
            get_settings,
            update_settings,
            install_hooks,
            uninstall_hooks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
