use std::{
    mem::take,
    ops::Deref,
    path::Path,
    sync::{Arc, Weak},
};

use anyhow::Result;
use log::warn;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

use crate::{
    core::{
        entities::{settings::Settings, yp_config::YPConfig},
        utils::failure::Failure,
    },
    features::{
        bbs::BbsListenerContainer,
        files::{
            settings::{
                load_settings_and_show_dialog_if_error, save_settings_and_show_dialog_if_error,
            },
            yp_configs::read_yp_configs_and_show_dialog_if_error,
        },
        hidden_features::{external_channels::ExternalChannels, stream_redirect::StreamRedirect},
        logger::LoggerController,
        peercast::broadcasting::Broadcasting,
        rtmp::rtmp_server::RtmpServer,
        terms_check::check_expired_terms,
        ui::Ui,
    },
};

use super::{
    app_bbs_listener_delegate::AppBbsListenerDelegate,
    app_rtmp_listener_delegate::AppRtmpListenerDelegate,
    app_ui_delegate::AppUiDelegate,
    entities::settings::{ChannelContent, ChannelSettings, Hidden},
};

async fn listen_rtmp_if_need(
    app: &App,
    app_rtmp_listener_delegate: Weak<AppRtmpListenerDelegate>,
) -> bool {
    let mut rtmp_server = app.rtmp_server.lock().await;
    rtmp_server.set_delegate(app_rtmp_listener_delegate);
    app.listen_rtmp_if_need(&mut rtmp_server, app.settings.lock().await.deref())
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
    pub ui: Ui,
    pub rtmp_server: Mutex<RtmpServer>,
    pub broadcasting: Mutex<Broadcasting>,
    pub bbs_listener_container: std::sync::Mutex<BbsListenerContainer>,
    pub logger_controller: LoggerController,
    external_channels: Mutex<Option<ExternalChannels>>,
    _app_bbs_listener_delegate: OnceCell<Arc<AppBbsListenerDelegate>>,
    _app_rtmp_listener_delegate: OnceCell<Arc<AppRtmpListenerDelegate>>,
}

impl App {
    async fn internal_new(app_dir: &Path, resource_dir: &Path) -> Self {
        Self {
            yp_configs: read_yp_configs_and_show_dialog_if_error(app_dir, resource_dir).await,
            settings: Mutex::new(load_settings_and_show_dialog_if_error(app_dir).await),
            ui: Ui::new(),
            rtmp_server: Mutex::new(RtmpServer::new()),
            broadcasting: Mutex::new(Broadcasting::new()),
            bbs_listener_container: std::sync::Mutex::new(BbsListenerContainer::new()),
            logger_controller: LoggerController::new(),
            external_channels: Default::default(),
            _app_bbs_listener_delegate: OnceCell::new(),
            _app_rtmp_listener_delegate: OnceCell::new(),
        }
    }

    pub async fn new(app_config_dir: &Path, resource_dir: &Path) -> Arc<App> {
        let zelf = Arc::new(Self::internal_new(app_config_dir, resource_dir).await);
        let app_rtmp_listener_delegate = Arc::new(AppRtmpListenerDelegate::new(
            Arc::downgrade(&zelf),
            app_config_dir.to_owned(),
        ));
        let app_ui_delegate = Arc::new(AppUiDelegate::new(
            Arc::downgrade(&zelf),
            app_config_dir.to_owned(),
        ));
        let app_bbs_listener_delegate =
            Arc::new(AppBbsListenerDelegate::new(Arc::downgrade(&zelf)));

        {
            let url = zelf.settings.lock().await.channel_settings.contact_url[0].clone();
            let mut bbs_listener_container = zelf.bbs_listener_container.lock().unwrap();
            let weak = Arc::downgrade(&app_bbs_listener_delegate);
            bbs_listener_container.set_delegate(weak);
            bbs_listener_container.set_url(url);
        }
        zelf._app_bbs_listener_delegate
            .set(app_bbs_listener_delegate)
            .unwrap_or_else(|_| panic!());
        {
            let app_ui_delegate = app_ui_delegate.clone();
            zelf.logger_controller
                .set_on_error(Box::new(move |failure| {
                    app_ui_delegate.on_error_log_controller(&failure);
                }));
        }

        let weak = Arc::downgrade(&app_rtmp_listener_delegate);
        let initial_rtmp = if listen_rtmp_if_need(&zelf, weak).await {
            "listening"
        } else {
            "idle"
        };
        zelf._app_rtmp_listener_delegate
            .set(app_rtmp_listener_delegate)
            .unwrap_or_else(|_| panic!());

        {
            let settings = zelf.settings.lock().await;
            if let Some(hidden) = &settings.other_settings.hidden {
                if let Some(stream_redirect_port) = hidden.stream_redirect_port {
                    StreamRedirect::default()
                        .run(
                            stream_redirect_port,
                            settings.general_settings.peer_cast_port,
                            settings.general_settings.channel_name[0].clone(),
                        )
                        .await;
                }
            };
        }

        let initial_channel_name =
            zelf.settings.lock().await.general_settings.channel_name[0].clone();
        let weak = Arc::downgrade(&app_ui_delegate);
        zelf.ui
            .prepare_ui(initial_rtmp.to_owned(), initial_channel_name, weak);

        zelf
    }

    pub async fn show_check_again_terms_dialog_if_expired(&self, app_dir: &Path) -> bool {
        let mut settings = self.settings.lock().await;
        match check_expired_terms(&self.yp_configs, &mut settings).await {
            Ok(true) => true,
            Ok(false) => {
                save_settings_and_show_dialog_if_error(app_dir, &settings).await;
                self.ui.reset_yp_terms(&settings);
                false
            }
            Err(e) => {
                warn!("{}", e);
                let warn = Failure::Warn("YP の利用規約の確認に失敗しました。".to_owned());
                self.ui.notify_failure(&warn);

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
                let ui = &self.ui;
                ui.notify_failure(&Failure::Error(err.to_string()));
                ui.set_rtmp("idle".to_owned());
                false
            }
            Ok(listening) => {
                let status = if listening { "listening" } else { "idle" };
                self.ui.set_rtmp(status.to_owned());
                listening
            }
        }
    }

    pub async fn update_channel(&self, broadcasting: &Broadcasting, settings: &Settings) {
        let res = broadcasting.update(&self.yp_configs, settings).await;
        if let Some(err) = res.err() {
            self.ui.notify_failure(&err);
        }
    }

    pub fn update_histories(&self, settings: &mut Settings, ui: &Ui) {
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
        ui.push_settings(settings);
    }

    pub async fn apply_channel_settings_to_external_channels(
        &self,
        hidden: &mut Hidden,
        channel_settings: &ChannelSettings,
    ) -> Result<()> {
        let mut external_channels = self.external_channels.lock().await;
        if external_channels.is_none() {
            *external_channels = Some(ExternalChannels::new(hidden));
        }
        external_channels
            .as_mut()
            .unwrap()
            .apply_channel_settings(hidden, channel_settings)
            .await
    }
}
