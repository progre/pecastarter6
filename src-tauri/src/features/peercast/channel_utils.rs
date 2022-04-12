use std::num::NonZeroU16;

use crate::{core::entities::settings::ChannelSettings, features::peercast::pecast_adapter::Info};

pub fn find_id(yp_list: &[(i32, String)], yp_host: &str) -> Option<i32> {
    let yp_host_pecast = format!("pcp://{}/", yp_host);
    yp_list
        .iter()
        .find(|(_, host)| host == &yp_host_pecast)
        .map(|&(id, _)| id)
}

pub fn info<'a>(
    channel_name: &'a str,
    genre: &'a str,
    channel_settings: &'a ChannelSettings,
) -> Info<'a> {
    Info {
        name: channel_name,
        url: &channel_settings.contact_url[0],
        bitrate: "",
        mime_type: "FLV",
        genre,
        desc: &channel_settings.desc,
        comment: &channel_settings.comment[0],
    }
}

pub fn ipv6_channel_name<T>(channel_name: &str, ipv4_id: &Option<T>) -> String {
    format!(
        "{}{}",
        channel_name,
        if ipv4_id.is_none() { "" } else { " (IPv6)" }
    )
}

pub fn rtmp_source(rtmp_conn_port: NonZeroU16) -> (&'static str, String) {
    (
        "RTMP Source",
        format!("rtmp://localhost:{}/live/livestream", rtmp_conn_port),
    )
}

pub fn loopback(id: &str) -> (&'static str, String) {
    ("他のチャンネル", format!("loopback:{}", id))
}
