use std::{ops::Deref, sync::Arc};

use log::warn;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    core::{
        entities::{settings::Settings, yp_config::YPConfig},
        utils::failure::Failure,
    },
    features::{
        files::{
            settings::{
                load_settings_and_show_dialog_if_error, save_settings_and_show_dialog_if_error,
            },
            yp_configs::read_yp_configs_and_show_dialog_if_error,
        },
        logger::LoggerController,
        peercast::broadcasting::Broadcasting,
        rtmp::rtmp_server::RtmpServer,
        terms_check::check_expired_terms,
        ui::Ui,
    },
};

use super::app_delegate_impl::AppDelegateImpl;

async fn listen_rtmp_if_need(app: &App, app_delegate: &Arc<AppDelegateImpl>) -> bool {
    let mut rtmp_server = app.rtmp_server.lock().await;
    let weak = Arc::downgrade(app_delegate);
    rtmp_server.set_delegate(weak);
    app.listen_rtmp_if_need(&mut rtmp_server, app.settings.lock().await.deref())
        .await
}

async fn run_ui(
    app: &App,
    app_delegate: &Arc<AppDelegateImpl>,
    initial_rtmp: String,
) -> JoinHandle<()> {
    let initial_channel_name = app.settings.lock().await.general_settings.channel_name[0].clone();
    let weak = Arc::downgrade(app_delegate);
    app.ui
        .lock()
        .unwrap()
        .run(initial_rtmp, initial_channel_name, weak)
        .await
}

pub struct App {
    pub yp_configs: Vec<YPConfig>,
    pub settings: Mutex<Settings>,
    pub ui: std::sync::Mutex<Ui>,
    pub rtmp_server: Mutex<RtmpServer>,
    pub broadcasting: Mutex<Broadcasting>,
    pub logger_controller: LoggerController,
}

impl App {
    async fn new() -> Self {
        Self {
            yp_configs: read_yp_configs_and_show_dialog_if_error().await,
            settings: Mutex::new(load_settings_and_show_dialog_if_error().await),
            ui: std::sync::Mutex::new(Ui::new()),
            rtmp_server: Mutex::new(RtmpServer::new()),
            broadcasting: Mutex::new(Broadcasting::new()),
            logger_controller: LoggerController::new(),
        }
    }

    pub async fn run() {
        let zelf = Arc::new(Self::new().await);
        let app_delegate = Arc::new(AppDelegateImpl::new(Arc::downgrade(&zelf)));

        {
            let app_delegate = app_delegate.clone();
            zelf.logger_controller
                .set_on_error(Box::new(move |failure| {
                    app_delegate.on_error_log_controller(&failure);
                }));
        }

        let initial_rtmp = if listen_rtmp_if_need(&zelf, &app_delegate).await {
            "listening"
        } else {
            "idle"
        };

        run_ui(&zelf, &app_delegate, initial_rtmp.to_owned())
            .await
            .await // long long awaiting
            .unwrap();
    }

    pub async fn show_check_again_terms_dialog_if_expired(&self) -> bool {
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
                save_settings_and_show_dialog_if_error(&settings).await;
                self.ui.lock().unwrap().reset_yp_terms(settings.clone());
                false
            }
            Err(e) => {
                warn!("{}", e);
                let warn = Failure::Warn("YP の利用規約の確認に失敗しました。".to_owned());
                self.ui.lock().unwrap().notify_failure(&warn);

                true
            }
        }
    }

    pub async fn listen_rtmp_if_need(
        &self,
        rtmp_server: &mut RtmpServer,
        settings: &Settings,
    ) -> bool {
        match rtmp_server
            .listen_rtmp_if_need(&self.yp_configs, settings)
            .await
        {
            Err(err) => {
                let ui = self.ui.lock().unwrap();
                ui.notify_failure(&Failure::Error(err.to_string()));
                ui.set_rtmp("idle".to_owned());
                false
            }
            Ok(listening) => {
                let status = if listening { "listening" } else { "idle" };
                self.ui.lock().unwrap().set_rtmp(status.to_owned());
                listening
            }
        }
    }

    pub async fn update_channel(&self, settings: &Settings) {
        let broadcasting = self.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            let res = broadcasting.update(&self.yp_configs, settings).await;
            if let Some(err) = res.err() {
                self.ui.lock().unwrap().notify_failure(&err);
            }
        }
    }
}
