use std::{
    num::NonZeroU16,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Local, SecondsFormat};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    spawn,
    task::JoinHandle,
    time::{interval, Duration},
};

use crate::{core::utils::failure::Failure, features::peercast::pecast_adapter::PeCaStAdapter};

fn to_csv_column(column: &str) -> String {
    column.replace('"', "\"\"")
}

fn to_csv_line(
    local: DateTime<Local>,
    ipv4_listeners_relays: Option<(u32, u32)>,
    ipv6_listeners_relays: Option<(u32, u32)>,
    genre: &str,
    description: &str,
    comment: &str,
) -> String {
    format!(
        "{},{},{},{},{},{},{},{}\n",
        local.to_rfc3339_opts(SecondsFormat::Secs, true),
        ipv4_listeners_relays
            .map(|x| x.0.to_string())
            .unwrap_or_default(),
        ipv4_listeners_relays
            .map(|x| x.1.to_string())
            .unwrap_or_default(),
        ipv6_listeners_relays
            .map(|x| x.0.to_string())
            .unwrap_or_default(),
        ipv6_listeners_relays
            .map(|x| x.1.to_string())
            .unwrap_or_default(),
        to_csv_column(genre),
        to_csv_column(description),
        to_csv_column(comment)
    )
}

async fn put_line(line: &str, path: &str) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write(line.as_bytes()).await?;
    Ok(())
}

async fn put_listeners_relays(
    ipv4_listeners_relays: Option<(u32, u32)>,
    ipv6_listeners_relays: Option<(u32, u32)>,
    path: &str,
) -> anyhow::Result<()> {
    put_line(
        &to_csv_line(
            Local::now(),
            ipv4_listeners_relays,
            ipv6_listeners_relays,
            "",
            "",
            "",
        ),
        path,
    )
    .await
}

async fn find_listeners_relays(
    port: NonZeroU16,
    ipv4_channel_id: Option<&str>,
    ipv6_channel_id: Option<&str>,
) -> Result<(Option<(u32, u32)>, Option<(u32, u32)>), Failure> {
    let view_xml = PeCaStAdapter::new(port).view_xml().await?;
    log::trace!("{:?}", view_xml);
    let ipv4_listeners_relays = ipv4_channel_id
        .map(|channel_id| view_xml.find_listeners_relays(channel_id))
        .flatten();
    let ipv6_listeners_relays = ipv6_channel_id
        .map(|channel_id| view_xml.find_listeners_relays(channel_id))
        .flatten();
    Ok((ipv4_listeners_relays, ipv6_listeners_relays))
}

async fn tick(
    peer_cast_port: &Mutex<NonZeroU16>,
    ipv4_channel_id: Option<&str>,
    ipv6_channel_id: Option<&str>,
    path: &str,
) -> Result<(), Failure> {
    let peer_cast_port = *peer_cast_port.lock().unwrap();
    let (ipv4, ipv6) =
        find_listeners_relays(peer_cast_port, ipv4_channel_id, ipv6_channel_id).await?;
    put_listeners_relays(ipv4, ipv6, path)
        .await
        .map_err(|err| {
            log::error!("{:?}", err);
            Failure::Error(err.to_string())
        })?;
    Ok(())
}

pub struct Logger {
    join_handle: JoinHandle<()>,
    path: String,
    peer_cast_port: Arc<std::sync::Mutex<NonZeroU16>>,
}

impl Logger {
    pub fn spawn(
        directory: &str,
        ipv4_channel_id: Option<String>,
        ipv6_channel_id: Option<String>,
        channel_name: &str,
        peer_cast_port: NonZeroU16,
        on_error: Box<dyn Send + Sync + Fn(Failure)>,
    ) -> Self {
        let path = format!(
            "{}/{}_{}.csv",
            directory,
            Local::now().format("%Y%m%d%H%M%S"),
            channel_name
        );
        let peer_cast_port = Arc::new(std::sync::Mutex::new(peer_cast_port));
        let join_handle = {
            let peer_cast_port = peer_cast_port.clone();
            let path = path.clone();
            spawn(async move {
                let mut interval = interval(Duration::from_secs(60));
                interval.tick().await;
                loop {
                    interval.tick().await;
                    match tick(
                        peer_cast_port.as_ref(),
                        ipv4_channel_id.as_deref(),
                        ipv6_channel_id.as_deref(),
                        &path,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => on_error(err),
                    }
                }
            })
        };
        Self {
            join_handle,
            path,
            peer_cast_port,
        }
    }

    pub fn set_peer_cast_port(&mut self, peer_cast_port: NonZeroU16) {
        *self.peer_cast_port.lock().unwrap() = peer_cast_port;
    }

    pub async fn put_info(&self, genre: &str, desc: &str, comment: &str) -> anyhow::Result<()> {
        put_line(
            &to_csv_line(Local::now(), None, None, genre, desc, comment),
            &self.path,
        )
        .await
    }

    pub fn abort(&mut self) {
        self.join_handle.abort();
    }
}
