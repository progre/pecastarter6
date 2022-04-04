use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_trait::async_trait;
use log::warn;
use tauri::api::dialog;
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle};

use crate::{
    core::{
        entities::{
            settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
            yp_config::YPConfig,
        },
        utils::{
            failure::Failure,
            tcp::{connect, pipe},
        },
    },
    features::{
        files::{
            settings::{
                load_settings_and_show_dialog_if_error, save_settings_and_show_dialog_if_error,
            },
            yp_configs::read_yp_configs_and_show_dialog_if_error,
        },
        peercast::broadcasting::Broadcasting,
        rtmp::{rtmp_server::RtmpServer, RtmpListenerDelegate},
        terms_check::check_expired_terms,
        ui::{Ui, UiDelegate},
    },
};

async fn listen_rtmp_if_need(app: &Arc<App>) -> bool {
    let mut rtmp_server = app.rtmp_server.lock().await;
    let weak = Arc::downgrade(app);
    rtmp_server.set_delegate(weak);
    app.listen_rtmp_if_need(&mut rtmp_server, app.settings.lock().await.deref())
        .await
}

async fn run_ui(app: &Arc<App>, initial_rtmp: String) -> JoinHandle<()> {
    let initial_channel_name = app.settings.lock().await.general_settings.channel_name[0].clone();
    let weak = Arc::downgrade(app);
    app.ui
        .lock()
        .await
        .run(initial_rtmp, initial_channel_name, weak)
        .await
}

fn show_file_error_dialog(message: &str) {
    let none: Option<&tauri::Window> = None;
    dialog::blocking::message(none, "Fatal", message);
}

pub struct App {
    yp_configs: Vec<YPConfig>,
    settings: Arc<Mutex<Settings>>,
    ui: Arc<Mutex<Ui>>,
    rtmp_server: Arc<Mutex<RtmpServer>>,
    broadcasting: Arc<Mutex<Broadcasting>>,
}

impl App {
    async fn new() -> Self {
        Self {
            yp_configs: read_yp_configs_and_show_dialog_if_error(show_file_error_dialog).await,
            settings: Arc::new(Mutex::new(
                load_settings_and_show_dialog_if_error(show_file_error_dialog).await,
            )),
            ui: Arc::new(Mutex::new(Ui::new())),
            rtmp_server: Arc::new(Mutex::new(RtmpServer::new())),
            broadcasting: Arc::new(Mutex::new(Broadcasting::new())),
        }
    }

    pub async fn run() {
        let zelf = Arc::new(Self::new().await);

        let initial_rtmp = if listen_rtmp_if_need(&zelf).await {
            "listening"
        } else {
            "idle"
        };

        run_ui(&zelf, initial_rtmp.to_owned()).await.await.unwrap(); // long long awaiting
    }

    async fn show_check_again_terms_dialog_if_expired(&self) -> bool {
        let (result, settings) = {
            let mut settings = self.settings.lock().await;
            (
                check_expired_terms(&self.yp_configs, &mut settings).await,
                settings.clone(),
            )
        };
        match result {
            Ok(true) => true,
            Ok(false) => {
                save_settings_and_show_dialog_if_error(&settings, show_file_error_dialog).await;
                self.ui.lock().await.reset_yp_terms(settings.clone());
                false
            }
            Err(e) => {
                warn!("{}", e);
                let warn = Failure::Warn("YP の利用規約の確認に失敗しました。".to_owned());
                self.ui.lock().await.notify_failure(&warn);

                true
            }
        }
    }

    async fn listen_rtmp_if_need(&self, rtmp_server: &mut RtmpServer, settings: &Settings) -> bool {
        let listening = rtmp_server.listen_rtmp_if_need(&self.yp_configs, settings);
        let status = if listening { "listening" } else { "idle" };
        self.ui.lock().await.set_rtmp(status.to_owned());
        listening
    }

    async fn update_channel(&self, settings: &Settings) {
        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&self.yp_configs, settings).await;
            if let Some(err) = res.err() {
                self.ui.lock().await.notify_failure(&err);
                return;
            }
        }

        save_settings_and_show_dialog_if_error(settings, show_file_error_dialog).await;
    }
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

#[async_trait]
impl UiDelegate for App {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings) {
        (self.yp_configs.clone(), self.settings.lock().await.clone())
    }

    async fn on_change_general_settings(&self, general_settings: GeneralSettings) {
        log::trace!("{:?}", general_settings);

        let mut settings = self.settings.lock().await;
        settings.general_settings = general_settings;

        self.listen_rtmp_if_need(self.rtmp_server.lock().await.deref_mut(), &settings)
            .await;

        self.update_channel(&settings).await;
    }

    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings) {
        log::trace!("{:?}", yellow_pages_settings);

        let mut settings = self.settings.lock().await;
        settings.yellow_pages_settings = yellow_pages_settings;

        self.listen_rtmp_if_need(self.rtmp_server.lock().await.deref_mut(), &settings)
            .await;

        self.update_channel(&settings).await;
    }

    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings) {
        log::trace!("{:?}", channel_settings);

        let mut settings = self.settings.lock().await;
        settings.channel_settings = channel_settings;

        self.update_channel(&settings).await;
    }
}

#[async_trait]
impl RtmpListenerDelegate for App {
    async fn on_connect(&self, incoming: TcpStream) {
        if !self.show_check_again_terms_dialog_if_expired().await {
            return;
        }

        let rtmp_conn_port = {
            let mut broadcasting = self.broadcasting.lock().await;
            let settings = self.settings.lock().await;
            let rtmp_conn_port = match broadcasting.broadcast(&self.yp_configs, &settings).await {
                Ok(ok) => ok,
                Err(err) => {
                    self.ui.lock().await.notify_failure(&err);
                    return;
                }
            };
            rtmp_conn_port
        };
        self.ui.lock().await.set_rtmp("streaming".to_owned());

        let outgoing = connect(&format!("localhost:{}", rtmp_conn_port)).await;
        pipe(incoming, outgoing).await; // long long awaiting

        self.ui.lock().await.set_rtmp("listening".to_owned());
        {
            let mut broadcasting = self.broadcasting.lock().await;
            let settings = self.settings.lock().await;
            match broadcasting
                .stop(settings.general_settings.peer_cast_port)
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    self.ui.lock().await.notify_failure(&err);
                    return;
                }
            }
        };
    }
}
