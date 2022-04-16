use std::{mem::take, ops::Deref, sync::Arc};

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

use super::{
    app_rtmp_listener_delegate::AppRtmpListenerDelegate, app_ui_delegate::AppUiDelegate,
    entities::settings::ChannelContent,
};

async fn listen_rtmp_if_need(
    app: &App,
    app_rtmp_listener_delegate: &Arc<AppRtmpListenerDelegate>,
) -> bool {
    let mut rtmp_server = app.rtmp_server.lock().await;
    let weak = Arc::downgrade(app_rtmp_listener_delegate);
    rtmp_server.set_delegate(weak);
    app.listen_rtmp_if_need(&mut rtmp_server, app.settings.lock().await.deref())
        .await
}

async fn run_ui(
    app: &App,
    app_ui_delegate: &Arc<AppUiDelegate>,
    initial_rtmp: String,
) -> JoinHandle<()> {
    let initial_channel_name = app.settings.lock().await.general_settings.channel_name[0].clone();
    let weak = Arc::downgrade(app_ui_delegate);
    app.ui
        .lock()
        .unwrap()
        .run(initial_rtmp, initial_channel_name, weak)
        .await
}

fn updated_value_with_history(history: Vec<String>, limit: usize) -> Vec<String> {
    let mut history_iter = history.into_iter();
    let working_value = history_iter.next().unwrap();
    let updated_history = [working_value.clone()]
        .into_iter()
        .chain(history_iter.filter(|x| x != &working_value))
        .filter(|x| !x.trim().is_empty())
        .take(limit);

    [working_value.clone()]
        .into_iter()
        .chain(updated_history)
        .collect()
}

fn updated_channel_content_history(
    history: Vec<ChannelContent>,
    genre: &str,
    desc: &str,
    limit: usize,
) -> Vec<ChannelContent> {
    let history_iter = history.into_iter();
    let working_value = ChannelContent {
        genre: genre.into(),
        desc: desc.into(),
    };
    [working_value.clone()]
        .into_iter()
        .chain(history_iter.filter(|x| x != &working_value))
        .filter(|x| !(x.genre.is_empty() && x.desc.is_empty()))
        .take(limit)
        .collect()
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
        let app_rtmp_listener_delegate =
            Arc::new(AppRtmpListenerDelegate::new(Arc::downgrade(&zelf)));
        let app_ui_delegate = Arc::new(AppUiDelegate::new(Arc::downgrade(&zelf)));

        {
            let app_ui_delegate = app_ui_delegate.clone();
            zelf.logger_controller
                .set_on_error(Box::new(move |failure| {
                    app_ui_delegate.on_error_log_controller(&failure);
                }));
        }

        let initial_rtmp = if listen_rtmp_if_need(&zelf, &app_rtmp_listener_delegate).await {
            "listening"
        } else {
            "idle"
        };

        run_ui(&zelf, &app_ui_delegate, initial_rtmp.to_owned())
            .await
            .await // long long awaiting
            .unwrap();
    }

    pub async fn show_check_again_terms_dialog_if_expired(&self) -> bool {
        let mut settings = self.settings.lock().await;
        match check_expired_terms(&self.yp_configs, &mut settings).await {
            Ok(true) => true,
            Ok(false) => {
                save_settings_and_show_dialog_if_error(&settings).await;
                self.ui.lock().unwrap().reset_yp_terms(&settings);
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

    pub async fn update_channel(&self, broadcasting: &Broadcasting, settings: &Settings) {
        let res = broadcasting.update(&self.yp_configs, settings).await;
        if let Some(err) = res.err() {
            self.ui.lock().unwrap().notify_failure(&err);
        }
    }

    pub fn update_histories(&self, settings: &mut Settings, ui: &std::sync::Mutex<Ui>) {
        settings.general_settings.channel_name =
            updated_value_with_history(take(&mut settings.general_settings.channel_name), 5);
        settings.channel_settings.channel_content_history = updated_channel_content_history(
            take(&mut settings.channel_settings.channel_content_history),
            &settings.channel_settings.genre,
            &settings.channel_settings.desc,
            20,
        );
        settings.channel_settings.comment =
            updated_value_with_history(take(&mut settings.channel_settings.comment), 20);
        settings.channel_settings.contact_url =
            updated_value_with_history(take(&mut settings.channel_settings.contact_url), 5);
        ui.lock().unwrap().push_settings(settings);
    }
}
