use std::sync::Arc;

use async_trait::async_trait;
use log::error;
use regex::Regex;
use tauri::api::{dialog, notification::Notification};
use tokio::{
    net::TcpStream,
    sync::{Mutex, MutexGuard},
};

use crate::{
    entities::settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
    failure::Failure,
    libs::broadcasting::Broadcasting,
    rtmp_listener::{RtmpListener, RtmpListenerDelegate},
    utils::tcp::{connect, find_free_port, pipe},
    window::{UiDelegate, Window},
};

#[derive(Clone)]
pub struct App {
    rtmp_listener: Arc<Mutex<RtmpListener>>,
    window: Arc<Mutex<Window>>,
    broadcasting: Arc<Mutex<Broadcasting>>,
    settings: Arc<Mutex<Settings>>,
}

impl App {
    pub async fn run() {
        let settings = Arc::new(Mutex::new(Settings::load().await));
        let rtmp_listener = Arc::new(Mutex::new(RtmpListener::new()));
        let window = Arc::new(Mutex::new(Window::new(settings.lock().await.clone())));
        let broadcasting = Arc::new(Mutex::new(Broadcasting::new()));

        let zelf = Arc::new(Self {
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
            rtmp_listener
                .spawn_listener(zelf.settings.lock().await.general_settings.rtmp_listen_port);
        }
        let window_join_handle = zelf.window.lock().await.run();
        window_join_handle.await.unwrap();
    }
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

fn normalize(yellow_pages_settings: YellowPagesSettings) -> YellowPagesSettings {
    let re = Regex::new(r"^(?:rtmp://)?(.*?)(?:/.*)?$").unwrap();
    YellowPagesSettings {
        ipv4_yp_host: re.captures(&yellow_pages_settings.ipv4_yp_host).unwrap()[1].to_owned(),
        ipv6_yp_host: re.captures(&yellow_pages_settings.ipv6_yp_host).unwrap()[1].to_owned(),
        ..yellow_pages_settings
    }
}

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
        let old_rtmp_listen_port = settings.general_settings.rtmp_listen_port;
        settings.general_settings = general_settings;

        let rtmp_port_updated = settings.general_settings.rtmp_listen_port != old_rtmp_listen_port;
        if rtmp_port_updated {
            self.rtmp_listener
                .lock()
                .await
                .spawn_listener(settings.general_settings.rtmp_listen_port);
        }

        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&settings).await;
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
        settings.yellow_pages_settings = normalize(yellow_pages_settings);

        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&settings).await;
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
            let res = broadcasting.update(&settings).await;
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
        let rtmp_conn_port = find_free_port().await.unwrap();
        {
            let mut broadcasting = self.broadcasting.lock().await;
            let settings = self.settings.lock().await;
            let res = broadcasting.broadcast(rtmp_conn_port, &settings).await;
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
