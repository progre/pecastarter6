use std::{
    ops::DerefMut,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use tokio::net::TcpStream;

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
        files::settings::save_settings_and_show_dialog_if_error, rtmp::RtmpListenerDelegate,
        ui::UiDelegate,
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

        self.app().update_channel(&settings).await;

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

        self.app()
            .listen_rtmp_if_need(self.app().rtmp_server.lock().await.deref_mut(), &settings)
            .await;

        self.app().update_channel(&settings).await;
    }

    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings) {
        log::trace!("{:?}", channel_settings);

        let app = self.app();
        let mut settings = app.settings.lock().await;
        settings.channel_settings = channel_settings;
        save_settings_and_show_dialog_if_error(&settings).await;

        self.app().update_channel(&settings).await;

        if let Err(err) = self
            .app()
            .logger_controller
            .on_change_channel_settings(&settings.channel_settings)
            .await
        {
            let failure = Failure::Warn(err.to_string());
            self.app().ui.lock().unwrap().notify_failure(&failure);
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

#[async_trait]
impl RtmpListenerDelegate for AppDelegateImpl {
    async fn on_connect(&self, incoming: TcpStream) {
        if !self.app().show_check_again_terms_dialog_if_expired().await {
            return;
        }

        let rtmp_conn_port = {
            let app = self.app();
            let settings = app.settings.lock().await;
            let (rtmp_conn_port, ipv4_id, ipv6_id) = {
                let mut broadcasting = app.broadcasting.lock().await;
                (
                    match broadcasting
                        .broadcast(&self.app().yp_configs, &settings)
                        .await
                    {
                        Err(err) => {
                            self.app().ui.lock().unwrap().notify_failure(&err);
                            return;
                        }
                        Ok(ok) => ok,
                    },
                    broadcasting.ipv4_id().clone(),
                    broadcasting.ipv6_id().clone(),
                )
            };
            if let Err(err) = self
                .app()
                .logger_controller
                .on_broadcast(ipv4_id, ipv6_id, &settings)
                .await
            {
                let failure = Failure::Warn(err.to_string());
                self.app().ui.lock().unwrap().notify_failure(&failure);
            }

            rtmp_conn_port
        };
        self.app()
            .ui
            .lock()
            .unwrap()
            .set_rtmp("streaming".to_owned());

        let outgoing = connect(&format!("localhost:{}", rtmp_conn_port)).await;
        pipe(incoming, outgoing).await; // long long awaiting

        self.app()
            .ui
            .lock()
            .unwrap()
            .set_rtmp("listening".to_owned());
        {
            if let Err(err) = self.app().logger_controller.on_stop_channel().await {
                let failure = Failure::Warn(err.to_string());
                self.app().ui.lock().unwrap().notify_failure(&failure);
            }

            let peer_cast_port = self
                .app()
                .settings
                .lock()
                .await
                .general_settings
                .peer_cast_port;
            if let Err(err) = self
                .app()
                .broadcasting
                .lock()
                .await
                .stop(peer_cast_port)
                .await
            {
                self.app().ui.lock().unwrap().notify_failure(&err);
            }
        };
    }
}
