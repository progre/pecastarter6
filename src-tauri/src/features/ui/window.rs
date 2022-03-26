use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use serde_json::{json, Value};
use tauri::{generate_context, generate_handler, Manager};
use tokio::{spawn, sync::mpsc, task::JoinHandle};

use crate::{
    core::entities::{
        settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
        yp_config::YPConfig,
    },
    features::terms_check,
};

#[async_trait]
pub trait WindowDelegate {
    fn on_load_page(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

#[tauri::command]
async fn fetch_hash(url: String) -> Result<String, String> {
    terms_check::fetch_hash(&url)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn initial_data(
    state: tauri::State<'_, WindowState>,
) -> Result<(Vec<YPConfig>, Settings), ()> {
    Ok(state.delegate().initial_data().await)
}

#[tauri::command]
async fn set_general_settings(
    general_settings: GeneralSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_general_settings(general_settings)
        .await;

    // WTF: 戻り値Resultが必須。バグ？
    Ok(())
}

#[tauri::command]
async fn set_yellow_pages_settings(
    yellow_pages_settings: YellowPagesSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_yellow_pages_settings(yellow_pages_settings)
        .await;

    Ok(())
}

#[tauri::command]
async fn set_channel_settings(
    channel_settings: ChannelSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_channel_settings(channel_settings)
        .await;

    Ok(())
}

trait StateExt {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate>;
}

impl StateExt for tauri::State<'_, WindowState> {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate> {
        self.delegate.upgrade().unwrap()
    }
}

pub struct Title {
    pub rtmp: String,
    pub channel_name: String,
}

unsafe impl Send for Title {}
unsafe impl Sync for Title {}

enum Command {
    PushSettings(Box<PushSettingsCommand>),
    Notify(NotifyCommand),
    Status(StatusCommand),
    UpdateTitle(UpdateTitleCommand),
}

struct PushSettingsCommand(Settings);
struct NotifyCommand(Value);
struct StatusCommand(Value);
struct UpdateTitleCommand(String);

struct WindowState {
    delegate: Weak<DynSendSyncWindowDelegate>,
}

pub struct Window {
    tx: mpsc::Sender<Command>,
    rx: Mutex<Option<mpsc::Receiver<Command>>>,
}

impl Window {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(16);
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }

    pub fn run(&self, delegate: Weak<DynSendSyncWindowDelegate>) -> JoinHandle<()> {
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        spawn(async move {
            let app = tauri::Builder::default()
                .manage(WindowState { delegate })
                .invoke_handler(generate_handler![
                    fetch_hash,
                    initial_data,
                    set_general_settings,
                    set_yellow_pages_settings,
                    set_channel_settings,
                ])
                .on_page_load(move |window, _page_load_payload| {
                    (window.state() as tauri::State<'_, WindowState>)
                        .delegate()
                        .on_load_page();
                })
                .any_thread()
                .build(generate_context!())
                .expect("error while running tauri application");
            log::trace!("run");
            app.run(move |app_handle, _| {
                if let Ok(command) = rx.try_recv() {
                    match command {
                        Command::PushSettings(settings_command) => {
                            app_handle
                                .emit_all("push_settings", settings_command.0)
                                .unwrap();
                        }
                        Command::Notify(NotifyCommand(json)) => {
                            app_handle.emit_all("notify", json).unwrap();
                        }
                        Command::Status(StatusCommand(json)) => {
                            app_handle.emit_all("status", json).unwrap();
                        }
                        Command::UpdateTitle(UpdateTitleCommand(title_status)) => {
                            log::trace!("receive send {}", title_status);
                            app_handle
                                .get_window("main")
                                .unwrap()
                                .set_title(&format!(
                                    "{} {}",
                                    app_handle.package_info().name,
                                    title_status,
                                ))
                                .unwrap()
                        }
                    }
                }
            });
        })
    }

    pub async fn push_settings(&self, settings: Settings) {
        self.tx
            .send(Command::PushSettings(Box::new(PushSettingsCommand(
                settings,
            ))))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub async fn notify(&self, level: &str, message: &str) {
        let json = json!({
            "level": level,
            "message": message
        });
        self.tx
            .send(Command::Notify(NotifyCommand(json)))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub async fn status(&self, rtmp: &str) {
        let json = json!({ "rtmp": rtmp });
        self.tx
            .send(Command::Status(StatusCommand(json)))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub fn update_title(&self, title_status: String) {
        self.tx
            .try_send(Command::UpdateTitle(UpdateTitleCommand(title_status)))
            .unwrap_or_else(|err| panic!("{}", err));
    }
}
