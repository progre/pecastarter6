use std::num::NonZeroU16;

use getset::Getters;
use tokio::try_join;
use versions::Version;

use crate::{
    core::{
        entities::{
            settings::{EachYellowPagesSettings, Settings, YellowPagesSettings},
            yp_config::YPConfig,
        },
        utils::{failure::Failure, tcp::find_free_port},
    },
    features::peercast::{
        channel_utils::{find_id, info, ipv6_channel_name, loopback, rtmp_source},
        pecast_adapter::{Info, PeCaStAdapter, Track},
    },
};

const EMPTY_TRACK: Track = Track {
    name: "",
    creator: "",
    genre: "",
    album: "",
    url: "",
};

async fn add_yellow_pages(adapter: &PeCaStAdapter, yp_host: &str) -> Result<i32, Failure> {
    adapter
        .add_yellow_page("pcp", yp_host, &format!("pcp://{}", yp_host))
        .await
}

async fn get_or_add_yellow_pages(
    adapter: &PeCaStAdapter,
    yp_list: &[(i32, String)],
    yp_host: &str,
) -> Result<Option<i32>, Failure> {
    if yp_host.is_empty() {
        Ok(None)
    } else if let Some(yp_id) = find_id(yp_list, yp_host) {
        Ok(Some(yp_id))
    } else {
        add_yellow_pages(adapter, yp_host).await.map(Some)
    }
}

async fn prepare_yellow_pages(
    adapter: &PeCaStAdapter,
    yp_settings: &YellowPagesSettings,
) -> Result<(Option<i32>, Option<i32>), Failure> {
    log::trace!("get yp");
    let yp_list = adapter.get_yellow_pages().await?;
    log::trace!("get yp {:?}", yp_list);

    Ok(tokio::try_join!(
        get_or_add_yellow_pages(adapter, &yp_list, &yp_settings.ipv4.host),
        get_or_add_yellow_pages(adapter, &yp_list, &yp_settings.ipv6.host)
    )?)
}

fn genre(
    yp_configs: &[YPConfig],
    yp_settings: &EachYellowPagesSettings,
    base_genre: &str,
) -> String {
    yp_configs
        .iter()
        .find(|x| x.host == yp_settings.host)
        .unwrap()
        .genre_full_text(base_genre, yp_settings)
}

async fn broadcast<'a>(
    adapter: &PeCaStAdapter,
    yp_id: i32,
    (source_stream, source_uri): &(&'static str, String),
    network_type: &str,
    info: &'a Info<'a>,
) -> Result<String, Failure> {
    // WTF: IPv6 一時アドレスが正しく設定されないケースの対策
    adapter.check_ports().await?;
    adapter.get_external_ip_addresses().await?;
    adapter
        .broadcast_channel(
            Some(yp_id),
            source_uri,
            source_stream,
            "ASF(WMV or WMA)",
            info,
            &EMPTY_TRACK,
            network_type,
        )
        .await
}

#[derive(Getters)]
pub struct Broadcasting {
    #[getset(get = "pub")]
    ipv4_id: Option<String>,
    #[getset(get = "pub")]
    ipv6_id: Option<String>,
}

unsafe impl Send for Broadcasting {}
unsafe impl Sync for Broadcasting {}

impl Broadcasting {
    pub fn new() -> Self {
        Self {
            ipv4_id: None,
            ipv6_id: None,
        }
    }

    pub fn is_broadcasting(&self) -> bool {
        self.ipv4_id.is_some() || self.ipv6_id.is_some()
    }

    pub async fn fetch_version(&self, settings: &Settings) -> Result<Version, Failure> {
        let adapter = PeCaStAdapter::new(settings.general_settings.peer_cast_port);
        let agent_name = adapter.get_version_info().await?;
        let version = agent_name
            .split('/')
            .nth(1)
            .ok_or_else(|| Failure::Fatal("Invalid agentName format".to_owned()))?;
        Version::new(version).ok_or_else(|| Failure::Fatal("Invalid agentName format".to_owned()))
    }

    pub async fn broadcast(
        &mut self,
        yp_configs: &[YPConfig],
        settings: &Settings,
    ) -> Result<NonZeroU16, Failure> {
        let rtmp_conn_port = if settings.general_settings.peer_cast_rtmp_port != 0 {
            NonZeroU16::new(settings.general_settings.peer_cast_rtmp_port).unwrap()
        } else {
            find_free_port().await.unwrap()
        };

        let adapter = PeCaStAdapter::new(settings.general_settings.peer_cast_port);
        let (ipv4_yp_id, ipv6_yp_id) =
            prepare_yellow_pages(&adapter, &settings.yellow_pages_settings).await?;
        let ipv4_channel_name = &settings.general_settings.channel_name[0];
        let base_genre = &settings.channel_settings.genre;

        if let Some(ipv6_yp_id) = ipv6_yp_id {
            let stream = rtmp_source(rtmp_conn_port);
            let ipv6_channel_name = &ipv6_channel_name(ipv4_channel_name, &ipv4_yp_id) as &str;
            let genre = genre(yp_configs, &settings.yellow_pages_settings.ipv6, base_genre);
            let info = info(ipv6_channel_name, &genre, &settings.channel_settings);
            self.ipv6_id = Some(broadcast(&adapter, ipv6_yp_id, &stream, "ipv6", &info).await?);
        }
        if let Some(ipv4_yp_id) = ipv4_yp_id {
            let stream = if let Some(ipv6_id) = &self.ipv6_id {
                loopback(ipv6_id)
            } else {
                rtmp_source(rtmp_conn_port)
            };
            let genre = genre(yp_configs, &settings.yellow_pages_settings.ipv4, base_genre);
            let info = info(ipv4_channel_name, &genre, &settings.channel_settings);
            self.ipv4_id = Some(broadcast(&adapter, ipv4_yp_id, &stream, "ipv4", &info).await?);
        }
        Ok(rtmp_conn_port)
    }

    pub async fn update(
        &self,
        yp_configs: &[YPConfig],
        settings: &Settings,
    ) -> Result<(), Failure> {
        let adapter = PeCaStAdapter::new(settings.general_settings.peer_cast_port);
        let ipv4_channel_name = &settings.general_settings.channel_name[0];
        let base_genre = &settings.channel_settings.genre;
        try_join!(
            async {
                if let Some(yp_id) = &self.ipv6_id {
                    let ipv6_channel_name = &ipv6_channel_name(ipv4_channel_name, &self.ipv4_id);
                    let genre = genre(yp_configs, &settings.yellow_pages_settings.ipv6, base_genre);
                    let info = info(ipv6_channel_name, &genre, &settings.channel_settings);
                    adapter.set_channel_info(yp_id, &info, &EMPTY_TRACK).await?;
                }
                Ok(())
            },
            async {
                if let Some(yp_id) = &self.ipv4_id {
                    let genre = genre(yp_configs, &settings.yellow_pages_settings.ipv4, base_genre);
                    let info = info(ipv4_channel_name, &genre, &settings.channel_settings);
                    adapter.set_channel_info(yp_id, &info, &EMPTY_TRACK).await?;
                }
                Ok(())
            }
        )?;
        Ok(())
    }

    pub async fn stop(&mut self, port: NonZeroU16) -> Result<(), Failure> {
        try_join!(
            async {
                log::trace!("stop ipv6");
                if let Some(yp_id) = &self.ipv6_id {
                    PeCaStAdapter::new(port).stop_channel(yp_id).await?;
                    self.ipv6_id = None;
                }
                log::trace!("stop ipv6 done");
                Ok(())
            },
            async {
                log::trace!("stop ipv4");
                if let Some(yp_id) = &self.ipv4_id {
                    let res = PeCaStAdapter::new(port).stop_channel(yp_id).await;
                    match &res {
                        Ok(_) => {}
                        Err(Failure::Error(message)) => {
                            if message != "Channel not found" {
                                return res;
                            }
                        }
                        Err(_) => return res,
                    }
                    self.ipv4_id = None;
                }
                log::trace!("stop ipv4 done");
                Ok(())
            }
        )?;
        Ok(())
    }
}
