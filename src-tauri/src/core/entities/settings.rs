use std::{cmp::max, collections::HashMap, num::NonZeroU16};

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

fn at_least_one_value(list: Vec<String>) -> Vec<String> {
    if list.is_empty() {
        vec!["".into()]
    } else {
        list
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub peer_cast_port: NonZeroU16,
    #[serde(default)]
    pub peer_cast_rtmp_port: u16,
    pub channel_name: Vec<String>,
    pub rtmp_listen_port: NonZeroU16,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        GeneralSettings {
            peer_cast_port: NonZeroU16::new(7144u16).unwrap(),
            peer_cast_rtmp_port: 0,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelContent {
    pub genre: String,
    pub desc: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSettings {
    pub channel_content_history: Vec<ChannelContent>,
    pub genre: String,
    pub desc: String,
    pub comment: Vec<String>,
    pub contact_url: Vec<String>,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            channel_content_history: Default::default(),
            genre: Default::default(),
            desc: Default::default(),
            comment: vec!["".to_owned()],
            contact_url: vec!["".to_owned()],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredChannelSettings {
    pub genre: Vec<String>,
    pub desc: Vec<String>,
    pub comment: Vec<String>,
    pub contact_url: Vec<String>,
}

impl StoredChannelSettings {
    fn into_internal(self) -> ChannelSettings {
        let genre_len = self.genre.len() as i32;
        let desc_len = self.desc.len() as i32;

        // 長さをそろえる
        let mut genre_iter =
            self.genre
                .into_iter()
                .chain(vec!["".into(); max(0, desc_len - genre_len) as usize]);
        let mut desc_iter = self
            .desc
            .into_iter()
            .chain(vec!["".into(); max(0, genre_len - desc_len) as usize]);

        let genre = genre_iter.next().unwrap();
        let desc = desc_iter.next().unwrap();
        let channel_content_history: Vec<_> = genre_iter
            .zip(desc_iter)
            .map(|(x, y)| ChannelContent { genre: x, desc: y })
            .collect();
        ChannelSettings {
            channel_content_history,
            genre,
            desc,
            comment: at_least_one_value(self.comment),
            contact_url: at_least_one_value(self.contact_url),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoringChannelSettings<'a> {
    pub genre: Vec<&'a str>,
    pub desc: Vec<&'a str>,
    pub comment: &'a Vec<String>,
    pub contact_url: &'a Vec<String>,
}

impl<'a> From<&'a ChannelSettings> for StoringChannelSettings<'a> {
    fn from(settings: &'a ChannelSettings) -> Self {
        let mut genre: Vec<&str> = vec![&settings.genre];
        genre.append(
            &mut settings
                .channel_content_history
                .iter()
                .map(|content| &content.genre as &str)
                .collect(),
        );
        let mut desc: Vec<&str> = vec![&settings.desc];
        desc.append(
            &mut settings
                .channel_content_history
                .iter()
                .map(|content| &content.desc as &str)
                .collect(),
        );
        StoringChannelSettings {
            genre,
            desc,
            comment: &settings.comment,
            contact_url: &settings.contact_url,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hidden {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fedimovie_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fedimovie_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_redirect_port: Option<NonZeroU16>,
    #[serde(default)]
    pub jpnkn_bbs_auto_comment: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherSettings {
    pub log_enabled: bool,
    pub log_output_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<Hidden>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub general_settings: GeneralSettings,
    pub yellow_pages_settings: YellowPagesSettings,
    pub channel_settings: ChannelSettings,
    pub other_settings: OtherSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    pub general_settings: GeneralSettings,
    pub yellow_pages_settings: YellowPagesSettings,
    pub channel_settings: StoredChannelSettings,
    #[serde(default)]
    pub other_settings: OtherSettings,
}

impl StoredSettings {
    pub fn into_internal(mut self) -> Settings {
        self.general_settings.channel_name = at_least_one_value(self.general_settings.channel_name);
        Settings {
            general_settings: self.general_settings,
            yellow_pages_settings: self.yellow_pages_settings,
            channel_settings: self.channel_settings.into_internal(),
            other_settings: self.other_settings,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoringSettings<'a> {
    pub general_settings: &'a GeneralSettings,
    pub yellow_pages_settings: &'a YellowPagesSettings,
    pub channel_settings: StoringChannelSettings<'a>,
    pub other_settings: &'a OtherSettings,
}

impl<'a> From<&'a Settings> for StoringSettings<'a> {
    fn from(settings: &'a Settings) -> Self {
        Self {
            general_settings: &settings.general_settings,
            yellow_pages_settings: &settings.yellow_pages_settings,
            channel_settings: StoringChannelSettings::from(&settings.channel_settings),
            other_settings: &settings.other_settings,
        }
    }
}
