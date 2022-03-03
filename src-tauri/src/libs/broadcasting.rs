use std::num::NonZeroU16;

use crate::{
    entities::settings::{ChannelSettings, Settings},
    failure::Failure,
    libs::pecast_adapter::{Info, Track},
};

use super::pecast_adapter::PeCaStAdapter;

const EMPTY_TRACK: Track = Track {
    name: "",
    creator: "",
    genre: "",
    album: "",
    url: "",
};

pub struct Broadcasting {
    ipv4_id: Option<String>,
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

    pub async fn broadcast(
        &mut self,
        rtmp_conn_port: u16,
        settings: &Settings,
    ) -> Result<(), Failure> {
        let adapter = PeCaStAdapter::new(settings.general_settings.peer_cast_port);

        let yp_settings = &settings.yellow_pages_settings;
        let yp_id = if yp_settings.ipv4_yp_host.is_empty() {
            None
        } else {
            let yp_id = adapter
                .get_yellow_pages()
                .await?
                .into_iter()
                .find(|(_, host)| host == &yp_settings.ipv4_yp_host)
                .map(|(id, _)| id);
            match yp_id {
                Some(_) => yp_id,
                None => {
                    let id = adapter
                        .add_yellow_page(
                            "pcp",
                            &yp_settings.ipv4_yp_host,
                            &format!("pcp://{}", yp_settings.ipv4_yp_host),
                        )
                        .await?;
                    Some(id)
                }
            }
        };

        self.ipv4_id = Some(
            adapter
                .broadcast_channel(
                    yp_id,
                    &format!("rtmp://localhost:{}/live/livestream", rtmp_conn_port),
                    "RTMP Source",
                    "ASF(WMV or WMA)",
                    &Info {
                        name: &settings.general_settings.channel_name[0],
                        url: &settings.channel_settings.contact_url[0],
                        bitrate: "",
                        mime_type: "FLV",
                        genre: &format!(
                            "{}{}",
                            settings.yellow_pages_settings.ipv4_yp_genre_prefix,
                            settings.channel_settings.genre[0]
                        ),
                        desc: &settings.channel_settings.desc[0],
                        comment: &settings.channel_settings.comment[0],
                    },
                    &EMPTY_TRACK,
                    "ipv4",
                )
                .await?,
        );
        Ok(())
    }

    pub async fn update(
        &self,
        peer_cast_port: NonZeroU16,
        channel_name: &str,
        channel_settings: &ChannelSettings,
    ) -> Result<(), Failure> {
        let adapter = PeCaStAdapter::new(peer_cast_port);

        adapter
            .set_channel_info(
                self.ipv4_id.as_ref().unwrap(),
                &Info {
                    name: channel_name,
                    url: &channel_settings.contact_url[0],
                    bitrate: "",
                    mime_type: "FLV",
                    genre: &channel_settings.genre[0],
                    desc: &channel_settings.desc[0],
                    comment: &channel_settings.comment[0],
                },
                &EMPTY_TRACK,
            )
            .await
    }

    pub async fn stop(&mut self, port: NonZeroU16) -> Result<(), Failure> {
        PeCaStAdapter::new(port)
            .stop_channel(self.ipv4_id.as_ref().unwrap())
            .await?;
        self.ipv4_id = None;
        Ok(())
    }
}
