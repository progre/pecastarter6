use std::{
    path::PathBuf,
    process::Command,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{
    utils::assets::EmbeddedAssets, AppHandle, Context, Invoke, InvokeMessage, Manager,
    UserAttentionType,
};

use crate::{
    core::{
        entities::{
            contact_status::ContactStatus,
            settings::{
                ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
            },
            yp_config::YPConfig,
        },
        utils::{dialog::show_dialog, tcp::find_free_port},
    },
    features::terms_check,
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
    fn on_build_app(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings, ContactStatus);
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

fn build_app(
    context: Context<EmbeddedAssets>,
    app_dir: PathBuf,
    delegate: Weak<DynSendSyncWindowDelegate>,
) -> tauri::App {
    tauri::Builder::default()
        .manage(WindowState { delegate })
        .invoke_handler(move |Invoke { message, resolver }| {
            let app_dir = app_dir.clone();
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
                        let selector = payload.get("selector").and_then(|x| x.as_str());
                        resolver.resolve(terms_check::fetch_hash(url, selector).await.unwrap());
                    }
                    "find_free_port" => {
                        resolver.resolve(find_free_port().await.unwrap());
                    }
                    "open_app_dir" => {
                        let cmd = if cfg!(target_os = "macos") {
                            "open"
                        } else {
                            "explorer.exe"
                        };
                        Command::new(cmd)
                            .arg(app_dir.to_str().unwrap())
                            .output()
                            .unwrap();
                    }
                    _ => panic!("unknown command"),
                }
            });
        })
        .build(context)
        .map_err(|err| {
            let mut note = "";
            if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(err)) = &err {
                if err.to_string().contains("WebView2") {
                    note = "WebView2 ランタイムをインストールするとこのエラーが解決する可能性があります。"
                }
            }
            show_dialog(&format!(
                "アプリケーションの起動に失敗しました。{}({}) ",
                note,
                err
            ));
            err
        })
        .expect("error while running tauri application")
}

pub struct Window {
    app_handle: std::sync::Mutex<Option<AppHandle>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            app_handle: Default::default(),
        }
    }

    pub fn run(
        &self,
        context: Context<EmbeddedAssets>,
        app_dir: PathBuf,
        delegate: Weak<DynSendSyncWindowDelegate>,
    ) {
        let app = build_app(context, app_dir, delegate.clone());
        *self.app_handle.lock().unwrap() = Some(app.app_handle());
        if let Some(delegate) = delegate.upgrade() {
            delegate.on_build_app()
        }
        app.run(|_, _| {});
    }

    pub fn push_settings(&self, settings: &Settings) {
        self.send("push_settings", serde_json::to_value(settings).unwrap());
    }

    pub fn push_contact_status(&self, contact_status: &ContactStatus) {
        self.send(
            "push_contact_status",
            serde_json::to_value(contact_status).unwrap(),
        );
    }

    pub fn notify(&self, level: &str, message: &str) {
        let attention = match level {
            "fatal" => Some(UserAttentionType::Critical),
            "error" => Some(UserAttentionType::Informational),
            _ => None,
        };
        if let Some(attention) = attention {
            if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
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
        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
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
        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
            app_handle.emit_all(event, payload).unwrap();
        }
    }
}
