use std::sync::Weak;

use log::{error, warn};
use tauri::api::{dialog, notification::Notification};
use tokio::task::JoinHandle;

use crate::core::{entities::settings::Settings, utils::failure::Failure};

use super::{window::Window, UiDelegate};

pub struct Ui {
    window: Window,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            window: Window::new(),
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn UiDelegate + Send + Sync>) {
        self.window.set_delegate(delegate);
    }

    pub fn run(&mut self, initial_rtmp: String, initial_channel_name: String) -> JoinHandle<()> {
        self.window.run(initial_rtmp, initial_channel_name)
    }

    pub fn notify_failure(&self, failure: &Failure) {
        match failure {
            Failure::Warn(message) => {
                warn!("{:?}", failure);
                self.notify_warn(message);
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

    pub fn reset_yp_terms(&self, settings: &Settings) {
        self.window.push_settings(settings);
        self.notify_error("YP の利用規約が変更されました。再度確認してください。");
    }

    pub fn status(&self, rtmp: String) {
        self.window.status(rtmp);
    }

    fn notify_warn(&self, message: &str) {
        self.window.notify("warn", message);
    }

    fn notify_error(&self, message: &str) {
        self.window.notify("error", message);
    }

    #[allow(dead_code)]
    fn notify_fatal(&self, message: &str) {
        self.window.notify("fatal", message);
    }
}
