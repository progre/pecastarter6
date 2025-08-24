use std::{
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_trait::async_trait;

use crate::{
    core::{
        entities::{
            settings::{
                ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
            },
            yp_config::YPConfig,
        },
        utils::failure::Failure,
    },
    features::{files::settings::save_settings_and_show_dialog_if_error, ui::UiDelegate},
};

use super::{app::App, entities::contact_status::ContactStatus};

pub struct AppUiDelegate {
    app: Weak<App>,
    settings_path: PathBuf,
}

impl AppUiDelegate {
    pub fn new(app: Weak<App>, settings_path: PathBuf) -> Self {
        Self { app, settings_path }
    }

    fn app(&self) -> Arc<App> {
        self.app.upgrade().unwrap()
    }

    pub fn on_error_log_controller(&self, failure: &Failure) {
        self.app().ui.notify_failure(failure);
    }
}

#[async_trait]
impl UiDelegate for AppUiDelegate {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings, ContactStatus) {
        let app = self.app();
        let yp_configs = app.yp_configs.clone();
        let settings = app.settings.lock().await.clone();
        let contact_status = app.bbs_listener_container.lock().unwrap().contact_status();
        (yp_configs, settings, contact_status)
    }

    async fn on_change_general_settings(&self, general_settings: GeneralSettings) {
        log::trace!("{:?}", general_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.general_settings = general_settings;
        save_settings_and_show_dialog_if_error(&self.settings_path, &settings).await;

        self.app()
            .listen_rtmp_if_need(self.app().rtmp_server.lock().await.deref_mut(), &settings)
            .await;

        self.app()
            .logger_controller
            .on_change_general_settings(&settings.general_settings)
            .await;
    }

    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings) {
        log::trace!("{:?}", yellow_pages_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.yellow_pages_settings = yellow_pages_settings;
        save_settings_and_show_dialog_if_error(&self.settings_path, &settings).await;

        app.listen_rtmp_if_need(app.rtmp_server.lock().await.deref_mut(), &settings)
            .await;

        let broadcasting = app.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            app.update_channel(&broadcasting, &settings).await;
        }
    }

    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings) {
        log::trace!("{:?}", channel_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.channel_settings = channel_settings;

        // PeCa 以外へのチャンネル情報反映
        // let settings_2: &mut Settings = &mut settings;
        let Settings {
            other_settings,
            channel_settings,
            ..
        } = &mut settings as &mut Settings;
        if let Some(hidden) = &mut other_settings.hidden
            && let Err(err) = self
                .app()
                .apply_channel_settings_to_external_channels(hidden, channel_settings)
                .await
        {
            let failure = Failure::Warn(err.to_string());
            app.ui.notify_failure(&failure);
        }

        let broadcasting = app.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            app.update_channel(&broadcasting, &settings).await;
            app.update_histories(&mut settings, &app.ui);
        }

        save_settings_and_show_dialog_if_error(&self.settings_path, &settings).await;

        app.bbs_listener_container
            .lock()
            .unwrap()
            .set_url(settings.channel_settings.contact_url[0].to_owned());

        if let Err(err) = app
            .logger_controller
            .on_change_channel_settings(&settings.channel_settings)
            .await
        {
            let failure = Failure::Warn(err.to_string());
            app.ui.notify_failure(&failure);
        }
    }

    async fn on_change_other_settings(&self, other_settings: OtherSettings) {
        log::trace!("{:?}", other_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.other_settings = other_settings;
        save_settings_and_show_dialog_if_error(&self.settings_path, &settings).await;

        let (is_broadcasting, ipv4_id, ipv6_id) = {
            let broadcasting = app.broadcasting.lock().await;
            (
                broadcasting.is_broadcasting(),
                broadcasting.ipv4_id().clone(),
                broadcasting.ipv6_id().clone(),
            )
        };
        if let Err(err) = self
            .app()
            .logger_controller
            .on_change_other_settings(ipv4_id, ipv6_id, &settings, is_broadcasting)
            .await
        {
            let failure = Failure::Warn(err.to_string());
            self.app().ui.notify_failure(&failure);
        }
    }
}
