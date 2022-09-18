use std::{
    num::NonZeroU16,
    path::PathBuf,
    sync::{Arc, Weak},
    time::Duration,
};

use async_trait::async_trait;
use tokio::{net::TcpStream, sync::Mutex, time::sleep};
use versions::Version;

use crate::{
    core::{
        entities::{settings::Settings, yp_config::YPConfig},
        utils::{
            failure::Failure,
            tcp::{connect, pipe},
        },
    },
    features::{
        files::settings::save_settings_and_show_dialog_if_error, logger::LoggerController,
        peercast::broadcasting::Broadcasting, rtmp::RtmpListenerDelegate,
    },
};

use super::app::App;

pub struct AppRtmpListenerDelegate {
    app: Weak<App>,
    app_dir: PathBuf,
}

impl AppRtmpListenerDelegate {
    pub fn new(app: Weak<App>, app_dir: PathBuf) -> Self {
        Self { app, app_dir }
    }

    fn app(&self) -> Arc<App> {
        self.app.upgrade().unwrap()
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
impl RtmpListenerDelegate for AppRtmpListenerDelegate {
    async fn on_connect(&self, incoming: TcpStream) {
        let app = self.app();
        if !app
            .show_check_again_terms_dialog_if_expired(&self.app_dir)
            .await
        {
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
                app.ui.notify_failure(&err);
                return;
            }
        };

        {
            let mut settings = app.settings.lock().await;
            app.update_histories(&mut settings, &app.ui);
            save_settings_and_show_dialog_if_error(&self.app_dir, &settings).await;
        }

        app.ui.set_rtmp("streaming".to_owned());

        let outgoing = connect(&format!("localhost:{}", rtmp_conn_port)).await;
        pipe(incoming, outgoing).await; // long long awaiting

        let pecast_version = {
            let settings = app.settings.lock().await;
            let result = app.broadcasting.lock().await.fetch_version(&settings).await;
            match result {
                Ok(ok) => ok,
                Err(err) => {
                    app.ui.notify_failure(&err);
                    return;
                }
            }
        };
        if pecast_version < Version::new("3.1.0.0").unwrap() {
            // NOTE: If the channel is deleted within 3 seconds of the stream closed,
            //       a tcp listener on PeerCastStation will remain.
            //       https://github.com/kumaryu/peercaststation/issues/490
            sleep(Duration::from_secs(6)).await;
        }

        app.ui.set_rtmp("listening".to_owned());

        let result = {
            let settings = app.settings.lock().await;
            stop_channel(&app.broadcasting, &settings, &app.logger_controller).await
        };
        match result {
            Ok(_) => {}
            Err(err) => {
                app.ui.notify_failure(&err);
                return;
            }
        };
    }
}
