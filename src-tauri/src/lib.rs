// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
// use tauri::Listener;
// use tauri::Manager;
use log::*;
use serde_json::{json, Value};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, State, WindowEvent,
};
use tauri_plugin_store::StoreExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use tokio::{
    process::Child,
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
};
use ts_rs::TS;

mod ffmpeg;
mod macros;
mod publisher;
mod transcript;
mod utils;

pub use publisher::Publish;
pub use utils::{
    copy_assets, delete_files,
    errors::ProcessError,
    logging::init_logging,
    presets::{collect_presets, Preset},
    template::Template,
    update, Sources,
};

use ffmpeg::{probe::MediaProbe, worker};

#[cfg(target_os = "macos")]
const MACOS_PATH: &str = "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
struct Task {
    path: String,
    r#in: f64,
    out: f64,
    fade: bool,
    lufs: bool,
    #[serde(default)]
    transcript: Option<String>,
    #[serde(default)]
    probe: MediaProbe,
    presets: Vec<Preset>,
    #[serde(default)]
    template: Option<Template>,
    #[ts(type = "string")]
    #[serde_as(as = "NoneAsEmptyString")]
    target: Option<String>,
    #[serde(default)]
    target_subfolder: bool,
    #[serde(default)]
    publish: Option<Publish>,
    #[ts(type = "bool")]
    active: Arc<AtomicBool>,
    #[ts(type = "bool")]
    finished: Arc<AtomicBool>,
}

#[derive(Clone)]
struct AppState {
    run: Arc<AtomicBool>,
    sender: Sender<Task>,
    encoder: Arc<Mutex<Option<Child>>>,
    config: Arc<Mutex<Config>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
struct Config {
    pub copyright: String,
    pub lufs: LufsConfig,
    pub transcript_cmd: String,
    pub transcript_lang: Vec<LangConfig>,
    #[serde(default)]
    pub publish_preset: Option<String>,
    #[serde(default)]
    pub publisher: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
struct LangConfig {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, TS)]
#[ts(export, export_to = "backend.d.ts")]
struct LufsConfig {
    pub i: f64,
    pub lra: f64,
    pub tp: f64,
}

impl AppState {
    fn new(tx: Sender<Task>) -> Self {
        Self {
            run: Arc::new(AtomicBool::new(false)),
            sender: tx,
            encoder: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(Config {
                lufs: LufsConfig {
                    i: -17.0,
                    lra: 9.0,
                    tp: -1.0,
                },

                transcript_cmd: String::new(),
                ..Default::default()
            })),
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        let encoder = self.encoder.clone();
        self.run.store(false, Ordering::SeqCst);

        tokio::spawn(async move {
            if let Some(mut proc) = encoder.lock().await.take() {
                if let Err(e) = proc.kill().await {
                    eprintln!("Failed to kill process: {e:?}");
                }
                if let Err(e) = proc.wait().await {
                    eprintln!("Failed to wait for process: {e:?}");
                }
            }
        });
    }
}

#[tauri::command]
async fn presets_get(app: AppHandle) -> tauri::Result<Vec<Preset>> {
    let presets = collect_presets(&app)
        .await
        .map_err(|e| tauri::Error::AssetNotFound(e.to_string()))?;

    Ok(presets)
}

#[tauri::command]
async fn file_drop(mut task: Task) -> Result<Task, ProcessError> {
    let path = task.path.clone();
    task.probe = MediaProbe::new(&path).await?;

    let sources = Sources::new(&path).await?;

    match sources.template {
        Some(src) => task.template = Some(Template::new(&src).await?),
        None => task.template = None,
    };

    Ok(task)
}

#[tauri::command]
async fn task_start(state: State<'_, AppState>) -> Result<(), ProcessError> {
    state.run.store(true, Ordering::SeqCst);

    Ok(())
}

#[tauri::command]
async fn task_send(task: Task, state: State<'_, AppState>) -> Result<(), ProcessError> {
    state.sender.send(task).await?;

    Ok(())
}

#[tauri::command]
async fn task_cancel(task: Task, state: State<'_, AppState>) -> Result<(), ProcessError> {
    let encoder = state.encoder.clone();
    state.run.store(false, Ordering::SeqCst);

    if let Some(mut proc) = encoder.lock().await.take() {
        proc.kill().await?;
        proc.wait().await?;
    }

    *encoder.lock().await = None;

    for preset in task.presets {
        if let Some(path) = preset.output_path {
            match delete_files(&path).await {
                Ok(_) => info!("Delete unfinished file: {path:?}"),
                Err(e) => error!("{}", e.to_string()),
            };
        }
    }

    Ok(())
}

#[tauri::command]
async fn template_save(template: Template, path: &str) -> Result<(), ProcessError> {
    template.save(path).await?;

    Ok(())
}

#[tauri::command]
async fn save_preset(app: AppHandle, mut preset: Preset) -> Result<(), ProcessError> {
    let preset_path = app.path().app_data_dir()?.join("presets");

    if !preset_path.is_dir() {
        tokio::fs::create_dir(&preset_path).await?;
    }

    preset.save(&preset_path).await
}

#[tauri::command]
async fn save_config(app: AppHandle, state: State<'_, AppState>) -> Result<(), ProcessError> {
    let store = app.store("config.json")?;
    let mut config = state.config.lock().await;

    if let Some(Value::String(copyright)) = store.get("copyright") {
        config.copyright = copyright;
    }

    if let Some(l) = store.get("lufs") {
        config.lufs.i = l.get("i").and_then(|l| l.as_f64()).unwrap_or_default();
        config.lufs.lra = l.get("lra").and_then(|l| l.as_f64()).unwrap_or_default();
        config.lufs.tp = l.get("tp").and_then(|l| l.as_f64()).unwrap_or_default();
    }

    if let Some(Value::String(s)) = store.get("transcript_cmd") {
        config.transcript_cmd = s;
    }

    store.close_resource();

    Ok(())
}

#[tauri::command]
async fn load_config(app: AppHandle, state: State<'_, AppState>) -> Result<(), ProcessError> {
    let app_handle = app.app_handle();

    copy_assets(app_handle).await.expect("Copy assets");

    let store = app_handle.store("config.json").expect("Open config");
    let mut config = state.config.lock().await;

    if let Some(Value::String(copyright)) = store.get("copyright") {
        config.copyright = copyright;
    }

    if let Some(Value::String(publish_preset)) = store.get("publish_preset") {
        config.publish_preset = Some(publish_preset);
    }

    config.publisher = store.get("publisher");

    match store.get("lufs") {
        Some(l) => {
            config.lufs.i = l.get("i").and_then(|l| l.as_f64()).unwrap_or_default();
            config.lufs.lra = l.get("lra").and_then(|l| l.as_f64()).unwrap_or_default();
            config.lufs.tp = l.get("tp").and_then(|l| l.as_f64()).unwrap_or_default();
        }
        None => {
            store.set("lufs", json!({ "i": -17.0, "lra": 9.0, "tp": -1.0 }));
            store.save().expect("Save config");

            config.lufs.i = -17.0;
            config.lufs.lra = 9.0;
            config.lufs.tp = -1.0;
        }
    }

    match store.get("transcript_cmd") {
        Some(t) => {
            if let Value::String(s) = t {
                config.transcript_cmd = s;
            }
        }
        None => {
            store.set("transcript_cmd", "");
            store.save().expect("Save config");
        }
    }

    store.close_resource();

    Ok(())
}

#[tauri::command]
fn shutdown_system() -> Result<(), ProcessError> {
    system_shutdown::shutdown()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> tauri::Result<()> {
    let (tx, rx) = channel(5);

    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin({
            let mut w_state = tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::POSITION)
                .with_state_flags(StateFlags::VISIBLE);

            if cfg!(not(target_os = "linux")) {
                w_state = w_state.with_state_flags(StateFlags::SIZE);
            }

            w_state.build()
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new(tx))
        .setup(|app| {
            let app_handle = app.app_handle().clone();
            let app_handle_clone = app_handle.clone();
            let app_handle_clone2 = app_handle.clone();
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;
            let window = app.get_webview_window("main").unwrap();

            tokio::spawn(async move {
                update(app_handle_clone2).await.unwrap();
            });

            window
                .restore_state(StateFlags::POSITION)
                .expect("Restore window position");

            #[cfg(not(target_os = "linux"))]
            window
                .restore_state(StateFlags::SIZE)
                .expect("Restore window size");

            init_logging(app_handle.clone());

            tokio::spawn(async move {
                let state = app_handle_clone.state::<AppState>().to_owned();

                if let Err(e) = worker::run(app_handle_clone.clone(), state, rx).await {
                    error!("{e:?}");
                };
            });

            let _ = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { .. } => {
                let app_state = window.state::<AppState>();
                let encoder = app_state.encoder.clone();

                tokio::spawn(async move {
                    if let Some(mut proc) = encoder.lock().await.take() {
                        if let Err(e) = proc.kill().await {
                            eprintln!("Failed to kill process: {e:?}");
                        }
                        if let Err(e) = proc.wait().await {
                            eprintln!("Failed to wait for process: {e:?}");
                        }
                    }
                });
            }
            WindowEvent::Moved { .. } => {
                let app = window.app_handle();
                app.save_window_state(StateFlags::all())
                    .expect("Save window position");
            }
            WindowEvent::Resized { .. } => {
                let app = window.app_handle();
                app.save_window_state(StateFlags::all())
                    .expect("Save window position");
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            file_drop,
            presets_get,
            task_start,
            task_send,
            task_cancel,
            template_save,
            save_config,
            save_preset,
            shutdown_system,
            load_config,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
