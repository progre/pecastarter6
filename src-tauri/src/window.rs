use std::{
    mem::replace,
    sync::{Arc, Mutex, Weak},
};

use async_trait::async_trait;
use serde_json::json;
use tauri::{generate_context, generate_handler, AppHandle, Manager};
use tokio::{spawn, task::JoinHandle};

use crate::entities::settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings};

#[async_trait]
pub trait UiDelegate {
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

#[tauri::command]
fn initial_settings(state: tauri::State<'_, WindowState>) -> Settings {
    state.initial_settings.clone()
}

#[tauri::command]
async fn set_general_settings(
    general_settings: GeneralSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate
        .upgrade()
        .unwrap()
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
        .delegate
        .upgrade()
        .unwrap()
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
        .delegate
        .upgrade()
        .unwrap()
        .on_change_channel_settings(channel_settings)
        .await;

    Ok(())
}

struct WindowState {
    initial_settings: Settings,
    delegate: Weak<dyn UiDelegate + Send + Sync>,
}

pub struct Window {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    initial_settings: Option<Settings>,
    delegate: Option<Weak<dyn UiDelegate + Send + Sync>>,
}

impl Window {
    pub fn new(initial_settings: Settings) -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
            initial_settings: Some(initial_settings),
            delegate: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn UiDelegate + Send + Sync>) {
        self.delegate = Some(delegate);
    }

    pub fn run(&mut self) -> JoinHandle<()> {
        let settings = replace(&mut self.initial_settings, None).unwrap();
        let delegate = replace(&mut self.delegate, None).unwrap();
        spawn(async move {
            let app = tauri::Builder::default()
                .manage(WindowState {
                    initial_settings: settings,
                    delegate,
                })
                .invoke_handler(generate_handler![
                    initial_settings,
                    set_general_settings,
                    set_yellow_pages_settings,
                    set_channel_settings,
                ])
                .any_thread()
                .build(generate_context!())
                .expect("error while running tauri application");
            app.run(|_, _| {});
        })
    }

    pub fn notify_error(&self, message: &str) {
        self.notify("error", message);
    }

    #[allow(dead_code)]
    pub fn notify_fatal(&self, message: &str) {
        self.notify("fatal", message);
    }

    fn notify(&self, level: &str, message: &str) {
        self.app_handle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .emit_all(
                "notify",
                json!({
                    "level": level,
                    "message": message
                }),
            )
            .unwrap();
    }
}
