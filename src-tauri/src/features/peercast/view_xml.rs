use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Servent {
    // uptime: u64,
}

#[derive(Debug, Deserialize)]
struct Bandwidth {}

#[derive(Debug, Deserialize)]
struct Connections {}

#[derive(Debug, Deserialize)]
struct ChannelsRelayed {}

#[derive(Debug, Deserialize)]
struct Hits {
    // pub hosts: u32,
    listeners: u32,
    relays: u32,
}

#[derive(Debug, Deserialize)]
struct Relay {
    listeners: u32,
    relays: u32,
}

#[derive(Debug, Deserialize)]
struct Channel {
    id: String,
    hits: Hits,
    relay: Relay,
}

#[derive(Debug, Deserialize)]
struct ChannelsFound {
    #[serde(rename = "channel")]
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
pub struct ViewXml {
    // session: String,
    // servent: Servent,
    // connections: Connections,
    // channels_relayed: ChannelsRelayed,
    channels_found: ChannelsFound,
}

impl ViewXml {
    pub fn find_listeners_relays(&self, channel_id: &str) -> Option<(u32, u32)> {
        self.channels_found
            .channels
            .iter()
            .find(|x| x.id == channel_id)
            .map(|channel| {
                (
                    channel.hits.listeners + channel.relay.listeners,
                    channel.hits.relays + channel.relay.relays,
                )
            })
    }
}
