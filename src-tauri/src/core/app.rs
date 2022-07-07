use std::{
    mem::take,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use log::warn;
use tauri::{
    api::path::{app_dir, resource_dir},
    generate_context,
    utils::assets::EmbeddedAssets,
    Context, Env,
};
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
        logger::LoggerController,
        peercast::broadcasting::Broadcasting,
        rtmp::rtmp_server::RtmpServer,
        terms_check::check_expired_terms,
        ui::Ui,
    },
};

use super::{
    app_bbs_listener_delegate::AppBbsListenerDelegate,
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
    context: Context<EmbeddedAssets>,
    app_dir: PathBuf,
    app: &App,
    app_ui_delegate: &Arc<AppUiDelegate>,
    initial_rtmp: String,
) {
    let initial_channel_name = app.settings.lock().await.general_settings.channel_name[0].clone();
    let weak = Arc::downgrade(app_ui_delegate);
    app.ui
        .run(context, app_dir, initial_rtmp, initial_channel_name, weak);
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
}

impl App {
    async fn new(app_dir: &Path, resource_dir: &Path) -> Self {
        Self {
            yp_configs: read_yp_configs_and_show_dialog_if_error(app_dir, resource_dir).await,
            settings: Mutex::new(load_settings_and_show_dialog_if_error(app_dir).await),
            ui: Ui::new(),
            rtmp_server: Mutex::new(RtmpServer::new()),
            broadcasting: Mutex::new(Broadcasting::new()),
            bbs_listener_container: std::sync::Mutex::new(BbsListenerContainer::new()),
            logger_controller: LoggerController::new(),
        }
    }

    pub async fn run() {
        let context = generate_context!();

        let app_dir = app_dir(context.config()).unwrap();
        let resource_dir = resource_dir(context.package_info(), &Env::default()).unwrap();

        let zelf = Arc::new(Self::new(&app_dir, &resource_dir).await);
        let app_rtmp_listener_delegate = Arc::new(AppRtmpListenerDelegate::new(
            Arc::downgrade(&zelf),
            app_dir.clone(),
        ));
        let app_ui_delegate = Arc::new(AppUiDelegate::new(Arc::downgrade(&zelf), app_dir.clone()));
        let app_bbs_listener_delegate =
            Arc::new(AppBbsListenerDelegate::new(Arc::downgrade(&zelf)));

        {
            let url = zelf.settings.lock().await.channel_settings.contact_url[0].clone();
            let mut bbs_listener_container = zelf.bbs_listener_container.lock().unwrap();
            let weak = Arc::downgrade(&app_bbs_listener_delegate);
            bbs_listener_container.set_delegate(weak);
            bbs_listener_container.set_url(url);
        }
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

        run_ui(
            context,
            app_dir,
            &zelf,
            &app_ui_delegate,
            initial_rtmp.to_owned(),
        )
        .await; // long long awaiting
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
}
