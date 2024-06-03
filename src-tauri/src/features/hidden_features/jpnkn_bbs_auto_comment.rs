mod jpnkn_bbs;

use std::{sync::Arc, time::Duration};

use log::{trace, warn};
use regex::Regex;
use tokio::{spawn, task::JoinHandle, time::sleep};
use tokio_stream::StreamExt;

use crate::core::app::App;

pub struct JpnknBbsAutoComment {
    app: Arc<App>,
    join_handle: Option<JoinHandle<()>>,
}

impl JpnknBbsAutoComment {
    pub fn new(app: Arc<App>) -> Self {
        Self {
            app,
            join_handle: None,
        }
    }

    pub async fn on_broadcast(&mut self) {
        let app = self.app.clone();
        self.join_handle = Some(spawn(async move {
            let url = {
                let settings = app.settings.lock().await;
                let contact_url = settings.channel_settings.contact_url.first();
                contact_url.cloned().unwrap_or_else(|| "".into()).to_owned()
            };
            let Some(board_name) = Regex::new(r"https://bbs.jpnkn.com/([^/]+)/")
                .unwrap()
                .captures(&url)
                .and_then(|x| x.get(1))
                .map(|x| x.as_str().to_owned())
            else {
                return;
            };
            'outer: loop {
                let mut jpnkn_stream = jpnkn_bbs::jpnkn_new_message_stream(&board_name);
                loop {
                    let Some(res) = jpnkn_stream.next().await else {
                        warn!("jpnkn_stream end");
                        sleep(Duration::from_secs(3)).await;
                        continue 'outer;
                    };
                    let message = match res {
                        Ok(ok) => ok,
                        Err(e) => {
                            log::error!("jpnkn_bbs error: {}", e);
                            continue;
                        }
                    };
                    let mut settings = app.settings.lock().await.clone();
                    // settings.channel_settings.comment の銭湯を取得するか、insertする
                    let comment = settings.channel_settings.comment.get_mut(0).unwrap();
                    *comment = format!("{}{}", message, comment);
                    trace!("update comment: {}", comment);
                    app.broadcasting
                        .lock()
                        .await
                        .update(&app.yp_configs, &settings)
                        .await
                        .unwrap();
                }
            }
        }));
    }

    pub fn on_stop_channel(&mut self) {
        self.join_handle.as_ref().unwrap().abort();
        self.join_handle = None;
    }
}
