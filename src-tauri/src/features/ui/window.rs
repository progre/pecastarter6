use std::sync::{Arc, Weak};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{AppHandle, InvokeMessage, Manager, UserAttentionType};

use crate::core::{
    app::App,
    entities::{
        contact_status::ContactStatus,
        settings::{
            ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
        },
        yp_config::YPConfig,
    },
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

pub struct WindowState {
    _app: Arc<App>,
    delegate: Weak<DynSendSyncWindowDelegate>,
}

impl WindowState {
    pub fn new(app: Arc<App>, delegate: Weak<DynSendSyncWindowDelegate>) -> Self {
        Self {
            _app: app,
            delegate,
        }
    }

    pub fn delegate(&self) -> Arc<DynSendSyncWindowDelegate> {
        self.delegate.upgrade().unwrap()
    }
}

#[async_trait]
pub trait InvokeMessageExt {
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

pub struct Window {
    app_handle: std::sync::Mutex<Option<AppHandle>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            app_handle: Default::default(),
        }
    }

    pub fn app_handle(&self) -> &std::sync::Mutex<Option<AppHandle>> {
        &self.app_handle
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
