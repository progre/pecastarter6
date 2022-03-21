use std::{
    mem::replace,
    sync::{Arc, Mutex, Weak},
};

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
pub trait UiDelegate {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

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
    Ok(state.delegate.upgrade().unwrap().initial_data().await)
}

#[tauri::command]
async fn set_general_settings(
    general_settings: GeneralSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state.title.lock().unwrap().channel_name = general_settings.channel_name[0].clone();

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

fn update_title(app_handle: &AppHandle) {
    let state = app_handle.state::<WindowState>();
    let title = state.title.lock().unwrap();
    let listening_icon = match title.rtmp.as_str() {
        "idle" => '×',
        "listening" => '○',
        "streaming" => '●',
        _ => unreachable!(),
    };
    app_handle
        .get_window("main")
        .unwrap()
        .set_title(&format!(
            "{} {}{}",
            app_handle.package_info().name,
            listening_icon,
            title.channel_name,
        ))
        .unwrap()
}

struct Title {
    rtmp: String,
    channel_name: String,
}

struct WindowState {
    delegate: Weak<dyn UiDelegate + Send + Sync>,
    title: Mutex<Title>,
}

pub struct Window {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    delegate: Option<Weak<dyn UiDelegate + Send + Sync>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
            delegate: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn UiDelegate + Send + Sync>) {
        self.delegate = Some(delegate);
    }

    pub fn run(&mut self, initial_rtmp: String, initial_channel_name: String) -> JoinHandle<()> {
        let delegate = replace(&mut self.delegate, None).unwrap();
        let app_handle = self.app_handle.clone();
        spawn(async move {
            let app = tauri::Builder::default()
                .manage(WindowState {
                    delegate,
                    title: Mutex::new(Title {
                        rtmp: initial_rtmp,
                        channel_name: initial_channel_name,
                    }),
                })
                .invoke_handler(generate_handler![
                    fetch_hash,
                    initial_data,
                    set_general_settings,
                    set_yellow_pages_settings,
                    set_channel_settings,
                ])
                .on_page_load(move |window, _page_load_payload| {
                    update_title(&window.app_handle());
                })
                .any_thread()
                .build(generate_context!())
                .expect("error while running tauri application");
            *app_handle.lock().unwrap() = Some(app.handle());
            app.run(|_, _| {});
        })
    }

    pub fn push_settings(&self, settings: &Settings) {
        self.app_handle
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .emit_all("push_settings", settings)
            .unwrap();
    }

    pub fn status(&self, rtmp: String) {
        let mut app_handle_opt = self.app_handle.lock().unwrap();
        let app_handle = match app_handle_opt.as_mut() {
            Some(some) => some,
            None => return,
        };

        let json = json!({ "rtmp": &rtmp });

        app_handle.state::<WindowState>().title.lock().unwrap().rtmp = rtmp;

        update_title(app_handle);

        app_handle.emit_all("status", json).unwrap();
    }

    pub fn notify(&self, level: &str, message: &str) {
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
