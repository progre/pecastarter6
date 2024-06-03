mod peer_tube;

use anyhow::Result;
use peer_tube::PeerTube;

use crate::core::entities::settings::{ChannelSettings, Hidden};

pub struct ExternalChannels {
    peer_tube: Option<PeerTube>,
}

impl ExternalChannels {
    pub fn new(hidden: &Hidden) -> Self {
        Self {
            peer_tube: { hidden.fedimovie_email.clone() }
                .zip(hidden.fedimovie_password.clone())
                .map(|(email, password)| PeerTube::new(email, password)),
        }
    }

    pub async fn apply_channel_settings(&mut self, settings: &ChannelSettings) -> Result<()> {
        if let Some(peer_tube) = &mut self.peer_tube {
            peer_tube.apply(settings).await?
        };

        Ok(())
    }
}
