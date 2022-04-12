use std::{
    num::NonZeroU16,
    ops::DerefMut,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{
    core::{
        entities::{
            settings::{
                ChannelSettings, GeneralSettings, OtherSettings, Settings, YellowPagesSettings,
            },
            yp_config::YPConfig,
        },
        utils::{
            failure::Failure,
            tcp::{connect, pipe},
        },
    },
    features::{
        files::settings::save_settings_and_show_dialog_if_error, logger::LoggerController,
        peercast::broadcasting::Broadcasting, rtmp::RtmpListenerDelegate, ui::UiDelegate,
    },
};

use super::app::App;

pub struct AppDelegateImpl(Weak<App>);

impl AppDelegateImpl {
    pub fn new(app: Weak<App>) -> Self {
        Self(app)
    }
}

impl AppDelegateImpl {
    fn app(&self) -> Arc<App> {
        self.0.upgrade().unwrap()
    }

    pub fn on_error_log_controller(&self, failure: &Failure) {
        self.app().ui.lock().unwrap().notify_failure(failure);
    }
}

#[async_trait]
impl UiDelegate for AppDelegateImpl {
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings) {
        (
            self.app().yp_configs.clone(),
            self.app().settings.lock().await.clone(),
        )
    }

    async fn on_change_general_settings(&self, general_settings: GeneralSettings) {
        log::trace!("{:?}", general_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.general_settings = general_settings;
        save_settings_and_show_dialog_if_error(&settings).await;

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
        save_settings_and_show_dialog_if_error(&settings).await;

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
        save_settings_and_show_dialog_if_error(&settings).await;

        let broadcasting = app.broadcasting.lock().await;
        if broadcasting.is_broadcasting() {
            app.update_channel(&broadcasting, &settings).await;
            app.update_histories(&mut settings, &app.ui);
        }

        if let Err(err) = app
            .logger_controller
            .on_change_channel_settings(&settings.channel_settings)
            .await
        {
            let failure = Failure::Warn(err.to_string());
            app.ui.lock().unwrap().notify_failure(&failure);
        }
    }

    async fn on_change_other_settings(&self, other_settings: OtherSettings) {
        log::trace!("{:?}", other_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.other_settings = other_settings;
        save_settings_and_show_dialog_if_error(&settings).await;

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
            self.app().ui.lock().unwrap().notify_failure(&failure);
        }
    }
}

async fn start_channel(
    broadcasting: &Mutex<Broadcasting>,
    yp_configs: &[YPConfig],
    settings: &Settings,
    logger_controller: &LoggerController,
) -> Result<NonZeroU16, Failure> {
    let mut broadcasting = broadcasting.lock().await;
    let rtmp_conn_port = broadcasting.broadcast(yp_configs, settings).await?;

    let ipv4_id = broadcasting.ipv4_id().clone();
    let ipv6_id = broadcasting.ipv6_id().clone();
    logger_controller
        .on_broadcast(ipv4_id, ipv6_id, settings)
        .await
        .map_err(|err| Failure::Warn(err.to_string()))?;

    Ok(rtmp_conn_port)
}

async fn stop_channel(
    broadcasting: &Mutex<Broadcasting>,
    settings: &Settings,
    logger_controller: &LoggerController,
) -> Result<(), Failure> {
    logger_controller
        .on_stop_channel()
        .await
        .map_err(|err| Failure::Warn(err.to_string()))?;

    let peer_cast_port = settings.general_settings.peer_cast_port;
    broadcasting.lock().await.stop(peer_cast_port).await
}

#[async_trait]
impl RtmpListenerDelegate for AppDelegateImpl {
    async fn on_connect(&self, incoming: TcpStream) {
        let app = self.app();
        if !app.show_check_again_terms_dialog_if_expired().await {
            return;
        }

        let result = {
            let settings = app.settings.lock().await;
            let lc = &app.logger_controller;
            start_channel(&app.broadcasting, &app.yp_configs, &settings, lc).await
        };
        let rtmp_conn_port = match result {
            Ok(ok) => ok,
            Err(err) => {
                app.ui.lock().unwrap().notify_failure(&err);
                return;
            }
        };

        app.update_histories(app.settings.lock().await.deref_mut(), &app.ui);

        app.ui.lock().unwrap().set_rtmp("streaming".to_owned());

        let outgoing = connect(&format!("localhost:{}", rtmp_conn_port)).await;
        pipe(incoming, outgoing).await; // long long awaiting

        app.ui.lock().unwrap().set_rtmp("listening".to_owned());

        let result = {
            let settings = app.settings.lock().await;
            stop_channel(&app.broadcasting, &settings, &app.logger_controller).await
        };
        match result {
            Ok(_) => {}
            Err(err) => {
                app.ui.lock().unwrap().notify_failure(&err);
                return;
            }
        };
    }
}
