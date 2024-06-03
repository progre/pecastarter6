mod peer_tube;

use anyhow::Result;
use peer_tube::PeerTube;

use crate::core::entities::settings::{ChannelSettings, Hidden};

pub struct ExternalChannels {
    peer_tube: PeerTube,
}

impl ExternalChannels {
    pub fn new(hidden: &Hidden) -> Self {
        Self {
            peer_tube: PeerTube::new(
                hidden.fedimovie_email.to_owned(),
                hidden.fedimovie_password.to_owned(),
            ),
        }
    }

    pub async fn apply_channel_settings(&mut self, settings: &ChannelSettings) -> Result<()> {
        self.peer_tube.apply(settings).await?;

        Ok(())
    }
}
