use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use log::{error, warn};
use tauri::api::{dialog, notification::Notification};
use tokio::task::JoinHandle;

use crate::core::{
    entities::{
        settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
        yp_config::YPConfig,
    },
    utils::failure::Failure,
};

use super::window::{Window, WindowDelegate};

#[async_trait]
pub trait UiDelegate {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

type DynSendSyncUiDelegate = dyn Send + Sync + UiDelegate;

pub struct Title {
    pub rtmp: String,
    pub channel_name: String,
}

impl ToString for Title {
    fn to_string(&self) -> String {
        let listening_icon = match self.rtmp.as_str() {
            "idle" => '×',
            "listening" => '○',
            "streaming" => '●',
            _ => unreachable!(),
        };
        format!("{}{}", listening_icon, self.channel_name)
    }
}

struct WindowDelegateImpl {
    pub window: Window,
    pub title: Mutex<Title>,
    ui_delegate: Weak<DynSendSyncUiDelegate>,
}

unsafe impl Send for WindowDelegateImpl {}
unsafe impl Sync for WindowDelegateImpl {}

impl WindowDelegateImpl {
    fn ui_delegate(&self) -> Arc<DynSendSyncUiDelegate> {
        self.ui_delegate.upgrade().unwrap()
    }
}

#[async_trait]
impl WindowDelegate for WindowDelegateImpl {
    async fn on_load_page(&self) {
        let title_status = self.title.lock().unwrap().to_string();
        self.window.set_title_status(title_status).await ;
    }

    async fn initial_data(&self) -> (Vec<YPConfig>, Settings) {
        self.ui_delegate().initial_data().await
    }

    async fn on_change_general_settings(&self, general_settings: GeneralSettings) {
        self.title.lock().unwrap().channel_name = general_settings.channel_name[0].clone();

        self.ui_delegate()
            .on_change_general_settings(general_settings)
            .await
    }

    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings) {
        self.ui_delegate()
            .on_change_yellow_pages_settings(yellow_pages_settings)
            .await
    }

    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings) {
        self.ui_delegate()
            .on_change_channel_settings(channel_settings)
            .await
    }
}

pub struct Ui {
    window_delegate_impl: Option<Arc<WindowDelegateImpl>>,
}

unsafe impl Send for Ui {}
unsafe impl Sync for Ui {}

impl Ui {
    pub fn new() -> Self {
        Self {
            window_delegate_impl: None,
        }
    }

    fn window(&self) -> Option<&Window> {
        self.window_delegate_impl.as_ref().map(|x| &x.window)
    }

    pub fn run(
        &mut self,
        initial_rtmp: String,
        initial_channel_name: String,
        delegate: Weak<DynSendSyncUiDelegate>,
    ) -> JoinHandle<()> {
        self.window_delegate_impl = Some(Arc::new(WindowDelegateImpl {
            title: Mutex::new(Title {
                rtmp: initial_rtmp,
                channel_name: initial_channel_name,
            }),
            window: Window::new(),
            ui_delegate: delegate,
        }));
        let weak = Arc::downgrade(self.window_delegate_impl.as_ref().unwrap());
        self.window().unwrap().run(weak)
    }

    pub async fn notify_failure(&self, failure: &Failure) {
        match failure {
            Failure::Warn(message) => {
                warn!("{:?}", failure);
                self.notify_warn(message).await;
            }
            Failure::Error(message) => {
                error!("{:?}", failure);
                Notification::default()
                    .title("Error")
                    .body(message)
                    .show()
                    .unwrap();
            }
            Failure::Fatal(message) => {
                error!("{:?}", failure);
                let none: Option<&tauri::Window> = None;
                dialog::blocking::message(none, "Fatal", message);
            }
        }
    }

    pub async fn reset_yp_terms(&self, settings: Settings) {
        if let Some(x) = self.window() {
            x.push_settings(settings).await;
        }
        self.notify_error("YP の利用規約が変更されました。再度確認してください。")
            .await;
    }

    pub async fn status(&self, rtmp: String) {
        if let Some(window_delegate_impl) = &self.window_delegate_impl {
            window_delegate_impl.window.set_rtmp(&rtmp).await;
            let title_status = {
                let title = &mut window_delegate_impl.title.lock().unwrap();
                title.rtmp = rtmp;
                title.to_string()
            };
            window_delegate_impl.window.set_title_status(title_status).await;
        }
    }

    async fn notify_warn(&self, message: &str) {
        if let Some(x) = self.window() {
            x.notify("warn", message).await
        }
    }

    async fn notify_error(&self, message: &str) {
        if let Some(x) = self.window() {
            x.notify("error", message).await
        }
    }

    #[allow(dead_code)]
    async fn notify_fatal(&self, message: &str) {
        if let Some(x) = self.window() {
            x.notify("fatal", message).await
        }
    }
}
