use std::{
    path::PathBuf,
    sync::{Arc, Weak},
};

use async_trait::async_trait;
use tokio::net::TcpStream;

use crate::{
    core::utils::{
        broadcast_events::{self, stop_broadcast},
        tcp::{connect, pipe},
    },
    features::rtmp::RtmpListenerDelegate,
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

        let (rtmp_conn_port, mut jpnkn_bbs_auto_comment) =
            match broadcast_events::start_broadcast(&app, &self.app_dir).await {
                Ok(ok) => ok,
                Err(err) => {
                    app.ui.notify_failure(&err);
                    return;
                }
            };

        let outgoing = match connect(&format!("localhost:{}", rtmp_conn_port)).await {
            Ok(ok) => ok,
            Err(err) => {
                app.ui.notify_failure(&err);
                return;
            }
        };
        pipe(incoming, outgoing).await; // long long awaiting

        match stop_broadcast(&app, jpnkn_bbs_auto_comment.as_mut()).await {
            Ok(_) => {}
            Err(err) => {
                app.ui.notify_failure(&err);
            }
        }
    }
}
