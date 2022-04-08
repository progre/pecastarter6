use std::{collections::HashMap, num::NonZeroU16};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PeerCastType {
    PeerCastOriginal,
    PeerCastStation,
}

impl Default for PeerCastType {
    fn default() -> Self {
        PeerCastType::PeerCastStation
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub peer_cast_port: NonZeroU16,
    #[serde(default)]
    peer_cast_rtmp_port: Option<u16>,
    pub channel_name: Vec<String>,
    pub rtmp_listen_port: NonZeroU16,
}

impl GeneralSettings {
    pub fn is_require_default_peer_cast_rtmp_port(&self) -> bool {
        self.peer_cast_rtmp_port.is_none()
    }

    pub fn peer_cast_rtmp_port(&self) -> u16 {
        self.peer_cast_rtmp_port.unwrap()
    }

    pub fn set_peer_cast_rtmp_port(&mut self, value: u16) {
        self.peer_cast_rtmp_port = Some(value);
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        GeneralSettings {
            peer_cast_port: NonZeroU16::new(7144u16).unwrap(),
            peer_cast_rtmp_port: None,
            channel_name: vec!["".to_owned()],
            rtmp_listen_port: NonZeroU16::new(1935u16).unwrap(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EachYellowPagesSettings {
    pub host: String,
    pub hide_listeners: bool,
    pub namespace: String,
    pub port_bandwidth_check: u8,
    pub no_log: bool,
    pub icon: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YellowPagesSettings {
    pub ipv4: EachYellowPagesSettings,
    pub ipv6: EachYellowPagesSettings,
    pub agreed_terms: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSettings {
    pub genre: Vec<String>,
    pub desc: Vec<String>,
    pub comment: Vec<String>,
    pub contact_url: Vec<String>,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        ChannelSettings {
            genre: vec!["".to_owned()],
            desc: vec!["".to_owned()],
            comment: vec!["".to_owned()],
            contact_url: vec!["".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherSettings {
    pub log_enabled: bool,
    pub log_output_directory: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub general_settings: GeneralSettings,
    pub yellow_pages_settings: YellowPagesSettings,
    pub channel_settings: ChannelSettings,
    #[serde(default)]
    pub other_settings: OtherSettings,
}

unsafe impl Send for Settings {}
unsafe impl Sync for Settings {}
