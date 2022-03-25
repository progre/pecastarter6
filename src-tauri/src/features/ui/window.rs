use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use serde_json::json;
use tauri::{generate_context, generate_handler, AppHandle, Manager};
use tokio::{spawn, task::JoinHandle};

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

struct WindowState {
    delegate: Weak<DynSendSyncWindowDelegate>,
}

pub struct Window {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn run(&self, delegate: Weak<DynSendSyncWindowDelegate>) -> JoinHandle<()> {
        let app_handle = self.app_handle.clone();
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
            *app_handle.lock().unwrap() = Some(app.handle());
            app.run(|_, _| {});
        })
    }

    fn app_handle(&self, callback: impl FnOnce(&AppHandle)) {
        callback(self.app_handle.lock().unwrap().as_ref().unwrap());
    }

    pub fn push_settings(&self, settings: &Settings) {
        self.app_handle(|app_handle| {
            app_handle.emit_all("push_settings", settings).unwrap();
        });
    }

    pub fn notify(&self, level: &str, message: &str) {
        self.app_handle(|app_handle| {
            let json = json!({
                "level": level,
                "message": message
            });
            app_handle.emit_all("notify", json).unwrap();
        });
    }

    pub fn status(&self, rtmp: &str) {
        self.app_handle(|app_handle| {
            let json = json!({ "rtmp": rtmp });

            app_handle.emit_all("status", json).unwrap();
        });
    }

    pub fn update_title(&self, title_status: &str) {
        self.app_handle(|app_handle| {
            app_handle
                .get_window("main")
                .unwrap()
                .set_title(&format!(
                    "{} {}",
                    app_handle.package_info().name,
                    title_status,
                ))
                .unwrap()
        });
    }
}
