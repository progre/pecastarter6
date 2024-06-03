mod jpnkn_bbs;

use std::{sync::Arc, time::Duration};

use log::{trace, warn};
use regex::Regex;
use tokio::{spawn, task::JoinHandle, time::sleep};
use tokio_stream::StreamExt;

use crate::core::app::App;

async fn apply_message(app: &App, msg: &str) {
    let msg = msg.replace("<br>", " ");
    const MAX_LEN: usize = 200;
    let msg = if msg.len() > MAX_LEN {
        msg.chars()
            .take(MAX_LEN - 1)
            .chain(['…'])
            .collect::<String>()
    } else {
        msg
    };

    let mut settings = app.settings.lock().await.clone();
    if let Some(comment) = settings.channel_settings.comment.first_mut() {
        *comment = format!("{}{}", msg, comment);
        trace!("update comment: {}", comment);
    } else {
        settings.channel_settings.comment.push(msg);
    }
    let broadcasting = app.broadcasting.lock().await;
    let result = broadcasting.update(&app.yp_configs, &settings).await;
    result.unwrap();
}

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
                    apply_message(&app, &message).await;
                }
            }
        }));
    }

    pub fn on_stop_channel(&mut self) {
        self.join_handle.as_ref().unwrap().abort();
        self.join_handle = None;
    }
}
