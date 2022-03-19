use std::{ops::DerefMut, sync::Arc};

use async_trait::async_trait;
use log::{error, warn};
use tauri::api::{dialog, notification::Notification};
use tokio::{
    net::TcpStream,
    sync::{Mutex, MutexGuard},
};

use crate::{
    entities::{
        settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
        yp_config::YPConfig,
    },
    failure::Failure,
    features::terms_check::check_expired_terms,
    libs::broadcasting::Broadcasting,
    rtmp_listener::{RtmpListener, RtmpListenerDelegate},
    utils::{
        read_yp_configs::read_yp_configs_and_show_dialog_if_error,
        tcp::{connect, find_free_port, pipe},
    },
    window::{UiDelegate, Window},
};

#[derive(Clone)]
pub struct App {
    yp_configs: Vec<YPConfig>,
    rtmp_listener: Arc<Mutex<RtmpListener>>,
    window: Arc<Mutex<Window>>,
    broadcasting: Arc<Mutex<Broadcasting>>,
    settings: Arc<Mutex<Settings>>,
}

impl App {
    pub async fn run() {
        let yp_configs = read_yp_configs_and_show_dialog_if_error().await;

        let settings = Arc::new(Mutex::new(Settings::load().await));
        let rtmp_listener = Arc::new(Mutex::new(RtmpListener::new()));
        let window = Arc::new(Mutex::new(Window::new(
            yp_configs.clone(),
            settings.lock().await.clone(),
        )));
        let broadcasting = Arc::new(Mutex::new(Broadcasting::new()));

        let zelf = Arc::new(Self {
            yp_configs,
            rtmp_listener,
            window,
            broadcasting,
            settings,
        });
        let weak = Arc::downgrade(&zelf);
        zelf.window.lock().await.set_delegate(weak);
        {
            let weak = Arc::downgrade(&zelf);
            let mut rtmp_listener = zelf.rtmp_listener.lock().await;
            rtmp_listener.set_delegate(weak);
            Self::listen_rtmp_if_need(
                rtmp_listener.deref_mut(),
                zelf.settings.lock().await.deref_mut(),
            );
        }
        let window_join_handle = zelf.window.lock().await.run();
        window_join_handle.await.unwrap();
    }

    fn listen_rtmp_if_need(rtmp_listener: &mut RtmpListener, settings: &Settings) {
        let running = rtmp_listener.port().is_some();
        let no_yp = settings.yellow_pages_settings.ipv4.host.is_empty()
            && settings.yellow_pages_settings.ipv6.host.is_empty();
        let changed_port = rtmp_listener.port().is_some()
            && rtmp_listener.port() != Some(settings.general_settings.rtmp_listen_port);
        if running && !changed_port {
            log::trace!("no change");
            return;
        }
        if running {
            log::trace!("stop_listener");
            rtmp_listener.stop_listener();
        }
        if no_yp {
            log::trace!("no wakeup");
            return;
        }
        rtmp_listener.spawn_listener(settings.general_settings.rtmp_listen_port);
    }

    async fn show_check_again_terms_dialog_if_expired(&self) -> bool {
        let mut settings = self.settings.lock().await;
        match check_expired_terms(&self.yp_configs, &mut settings).await {
            Ok(true) => true,
            Ok(false) => {
                let window = self.window.lock().await;
                window.push_settings(&settings);
                window.notify_error("YP の利用規約が変更されました。再度確認してください。");

                false
            }
            Err(e) => {
                warn!("{}", e);
                self.window
                    .lock()
                    .await
                    .notify_warn("YP の利用規約の確認に失敗しました。");

                true
            }
        }
    }
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

fn notify_failure(window: MutexGuard<Window>, failure: &Failure) {
    match failure {
        Failure::Warn(message) => {
            window.notify_error(message);
        }
        Failure::Error(message) => {
            Notification::default()
                .title("Error")
                .body(message)
                .show()
                .unwrap();
        }
        Failure::Fatal(message) => {
            let none: Option<&tauri::Window> = None;
            dialog::blocking::message(none, "Fatal", message);
        }
    }
}

#[async_trait]
impl UiDelegate for App {
    async fn on_change_general_settings(&self, general_settings: GeneralSettings) {
        log::trace!("{:?}", general_settings);

        let mut settings = self.settings.lock().await;
        settings.general_settings = general_settings;

        Self::listen_rtmp_if_need(self.rtmp_listener.lock().await.deref_mut(), &settings);

        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&self.yp_configs, &settings).await;
            if let Some(err) = res.err() {
                error!("{:?}", err);
                notify_failure(self.window.lock().await, &err);
                return;
            }
        }

        settings.save().await;
    }

    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings) {
        log::trace!("{:?}", yellow_pages_settings);

        let mut settings = self.settings.lock().await;
        settings.yellow_pages_settings = yellow_pages_settings;

        Self::listen_rtmp_if_need(self.rtmp_listener.lock().await.deref_mut(), &settings);

        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&self.yp_configs, &settings).await;
            if let Some(err) = res.err() {
                error!("{:?}", err);
                notify_failure(self.window.lock().await, &err);
                return;
            }
        }

        settings.save().await;
    }

    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings) {
        log::trace!("{:?}", channel_settings);

        let mut settings = self.settings.lock().await;
        settings.channel_settings = channel_settings;

        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&self.yp_configs, &settings).await;
            if let Some(err) = res.err() {
                error!("{:?}", err);
                notify_failure(self.window.lock().await, &err);
                return;
            }
        }

        settings.save().await;
    }
}

#[async_trait]
impl RtmpListenerDelegate for App {
    async fn on_connect(&self, incoming: TcpStream) {
        if !self.show_check_again_terms_dialog_if_expired().await {
            return;
        }

        let rtmp_conn_port = find_free_port().await.unwrap();
        {
            let mut broadcasting = self.broadcasting.lock().await;
            let settings = self.settings.lock().await;
            let res = broadcasting
                .broadcast(rtmp_conn_port, &self.yp_configs, &settings)
                .await;
            if let Some(err) = res.err() {
                error!("{:?}", err);
                notify_failure(self.window.lock().await, &err);
                return;
            }
        }

        let rtmp_conn_host = format!("localhost:{}", rtmp_conn_port);
        let outgoing = connect(&rtmp_conn_host).await;
        pipe(incoming, outgoing).await;

        {
            let mut broadcasting = self.broadcasting.lock().await;
            let settings = self.settings.lock().await;
            let res = broadcasting
                .stop(settings.general_settings.peer_cast_port)
                .await;
            if let Some(err) = res.err() {
                error!("{:?}", err);
                notify_failure(self.window.lock().await, &err);
                return;
            }
        }
    }
}
