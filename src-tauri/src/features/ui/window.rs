use std::{
    process::Command,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{
    generate_context, AppHandle, Invoke, InvokeMessage, Manager, PageLoadPayload, UserAttentionType,
};
use tokio::task::JoinHandle;

use crate::{
    core::{
        entities::{
            settings::{
                ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
            },
            yp_config::YPConfig,
        },
        utils::tcp::find_free_port,
    },
    features::{files::APP_DIR, terms_check},
};

/*
 * # tokio スレッドと std スレッドの間の通信について
 *
 * tokio スレッド -> std スレッド
 * * スレッドセーフ?の AppHandle に一任
 * * channel でのデータ転送
 *
 * std スレッド -> tokio スレッド
 * * delegate (但し即時処理のみ対応)
 * * channel でのデータ転送 (但し戻り値を持てない)
 * * tauri::async_runtime::spawn でラップ
 */

#[async_trait]
pub trait WindowDelegate {
    fn on_load_page(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
    async fn on_change_other_settings(&self, other_settings: OtherSettings);
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

struct WindowState {
    delegate: Weak<DynSendSyncWindowDelegate>,
}

impl WindowState {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate> {
        self.delegate.upgrade().unwrap()
    }
}

#[async_trait]
trait InvokeMessageExt {
    fn get_from_payload<T>(&self, param: &str) -> Option<T>
    where
        T: DeserializeOwned;
}

#[async_trait]
impl InvokeMessageExt for InvokeMessage {
    fn get_from_payload<T>(&self, param: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self.payload()
            .get(param)
            .map(|x| serde_json::from_value(x.clone()).unwrap())
    }
}

fn build_app(delegate: Weak<DynSendSyncWindowDelegate>) -> tauri::App {
    tauri::Builder::default()
        .manage(WindowState { delegate })
        .invoke_handler(|Invoke { message, resolver }| {
            tauri::async_runtime::spawn(async move {
                let delegate = message.state_ref().get::<WindowState>().delegate();
                match message.command() {
                    "initial_data" => {
                        resolver.resolve(delegate.initial_data().await);
                    }
                    "put_settings" => {
                        if let Some(settings) = message.get_from_payload("generalSettings") {
                            delegate.on_change_general_settings(settings).await;
                        }
                        if let Some(settings) = message.get_from_payload("yellowPagesSettings") {
                            delegate.on_change_yellow_pages_settings(settings).await;
                        }
                        if let Some(settings) = message.get_from_payload("channelSettings") {
                            delegate.on_change_channel_settings(settings).await;
                        }
                        if let Some(settings) = message.get_from_payload("otherSettings") {
                            delegate.on_change_other_settings(settings).await;
                        }
                    }
                    "fetch_hash" => {
                        let payload = message.payload();
                        let url = payload.get("url").unwrap().as_str().unwrap();
                        let selector = payload.get("selector").map(|x| x.as_str()).flatten();
                        resolver.resolve(terms_check::fetch_hash(url, selector).await.unwrap());
                    }
                    "find_free_port" => {
                        resolver.resolve(find_free_port().await.unwrap());
                    }
                    "open_app_dir" => {
                        Command::new("explorer.exe")
                            .arg(APP_DIR.to_str().unwrap())
                            .output()
                            .unwrap();
                    }
                    _ => panic!("unknown command"),
                }
            });
        })
        .on_page_load(|window: tauri::Window, _: PageLoadPayload| {
            let delegate = (window.state() as tauri::State<'_, WindowState>).delegate();
            delegate.on_load_page();
        })
        .any_thread()
        .build(generate_context!())
        .expect("error while running tauri application")
}

pub struct Window {
    app_handle: Option<AppHandle>,
}

impl Window {
    pub fn new() -> Self {
        Self { app_handle: None }
    }

    pub async fn run(&mut self, delegate: Weak<DynSendSyncWindowDelegate>) -> JoinHandle<()> {
        let (start_tx, start_rx) = tokio::sync::oneshot::channel();
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let app = build_app(delegate);
            start_tx.send(app.app_handle()).unwrap();
            app.run(|_, _| {});
            stop_tx.send(()).unwrap();
        });
        self.app_handle = Some(start_rx.await.unwrap());
        tokio::spawn(async {
            stop_rx.await.unwrap();
        })
    }

    pub fn push_settings(&self, settings: Settings) {
        self.send("push_settings", serde_json::to_value(settings).unwrap());
    }

    pub fn notify(&self, level: &str, message: &str) {
        let attention = match level {
            "fatal" => Some(UserAttentionType::Critical),
            "error" => Some(UserAttentionType::Informational),
            _ => None,
        };
        if let Some(attention) = attention {
            if let Some(app_handle) = &self.app_handle {
                app_handle
                    .get_window("main")
                    .unwrap()
                    .request_user_attention(Some(attention))
                    .unwrap();
            }
        }
        self.send(
            "notify",
            json!({
                "level": level,
                "message": message
            }),
        );
    }

    pub fn set_rtmp(&self, rtmp: &str) {
        self.send("status", json!({ "rtmp": rtmp }));
    }

    pub fn set_title_status(&self, title_status: String) {
        if let Some(app_handle) = &self.app_handle {
            app_handle
                .get_window("main")
                .unwrap()
                .set_title(&format!(
                    "{} {}",
                    app_handle.package_info().name,
                    title_status,
                ))
                .unwrap();
        }
    }

    fn send(&self, event: &str, payload: Value) {
        if let Some(app_handle) = &self.app_handle {
            app_handle.emit_all(event, payload).unwrap();
        }
    }
}
