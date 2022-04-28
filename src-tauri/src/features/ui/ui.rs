use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use log::{error, warn};
use tauri::api::dialog;

use crate::core::{
    entities::{
        contact_status::ContactStatus,
        settings::{
            ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
        },
        yp_config::YPConfig,
    },
    utils::failure::Failure,
};

use super::window::{Window, WindowDelegate};

#[async_trait]
pub trait UiDelegate {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings, ContactStatus);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
    async fn on_change_other_settings(&self, other_settings: OtherSettings);
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
    window: Weak<Window>,
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
    fn on_load_page(&self) {
        let title_status = self.title.lock().unwrap().to_string();
        self.window
            .upgrade()
            .unwrap()
            .set_title_status(title_status);
    }

    async fn initial_data(&self) -> (Vec<YPConfig>, Settings, ContactStatus) {
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

    async fn on_change_other_settings(&self, other_settings: OtherSettings) {
        self.ui_delegate()
            .on_change_other_settings(other_settings)
            .await
    }
}

pub struct Ui {
    window: Arc<Window>,
    window_delegate_impl: std::sync::Mutex<Option<Arc<WindowDelegateImpl>>>,
}

unsafe impl Send for Ui {}
unsafe impl Sync for Ui {}

impl Ui {
    pub fn new() -> Self {
        Self {
            window: Arc::new(Window::new()),
            window_delegate_impl: Default::default(),
        }
    }

    pub fn run(
        &self,
        initial_rtmp: String,
        initial_channel_name: String,
        delegate: Weak<DynSendSyncUiDelegate>,
    ) {
        *self.window_delegate_impl.lock().unwrap() = Some(Arc::new(WindowDelegateImpl {
            title: Mutex::new(Title {
                rtmp: initial_rtmp,
                channel_name: initial_channel_name,
            }),
            window: Arc::downgrade(&self.window),
            ui_delegate: delegate,
        }));
        let weak = Arc::downgrade(self.window_delegate_impl.lock().unwrap().as_ref().unwrap());
        self.window.run(weak);
    }

    pub fn notify_failure(&self, failure: &Failure) {
        match failure {
            Failure::Warn(message) => {
                warn!("{:?}", failure);
                self.notify_warn(message);
            }
            Failure::Error(message) => {
                error!("{:?}", failure);
                self.notify_error(message);
            }
            Failure::Fatal(message) => {
                error!("{:?}", failure);
                self.notify_fatal(message)
            }
        }
    }

    pub fn push_settings(&self, settings: &Settings) {
        self.window.push_settings(settings);
    }

    pub fn push_contact_status(&self, contact_status: &ContactStatus) {
        self.window.push_contact_status(contact_status);
    }

    pub fn reset_yp_terms(&self, settings: &Settings) {
        self.window.push_settings(settings);
        self.notify_error("YP の利用規約が変更されました。再度確認してください。");
    }

    pub fn set_rtmp(&self, rtmp: String) {
        if let Some(window_delegate_impl) = self.window_delegate_impl.lock().unwrap().as_ref() {
            self.window.set_rtmp(&rtmp);
            let title_status = {
                let title = &mut window_delegate_impl.title.lock().unwrap();
                title.rtmp = rtmp;
                title.to_string()
            };
            self.window.set_title_status(title_status);
        }
    }

    fn notify_warn(&self, message: &str) {
        self.window.notify("warn", message)
    }

    fn notify_error(&self, message: &str) {
        self.window.notify("error", message)
    }

    fn notify_fatal(&self, message: &str) {
        let none: Option<&tauri::Window> = None;
        dialog::blocking::message(none, "Fatal", message);
    }
}
