mod peer_tube;
mod restream;

use anyhow::Result;
use peer_tube::PeerTube;
use restream::Restream;

use crate::core::entities::settings::{ChannelSettings, Hidden};

pub struct ExternalChannels {
    peer_tube: Option<PeerTube>,
    restream: Option<Restream>,
}

impl ExternalChannels {
    pub fn new(hidden: &Hidden) -> Self {
        Self {
            peer_tube: { hidden.fedimovie_email.clone() }
                .zip(hidden.fedimovie_password.clone())
                .map(|(email, password)| PeerTube::new(email, password)),
            restream: { hidden.restream_client_id.as_ref() }
                .zip(hidden.restream_client_secret.as_ref())
                .map(|(restream_client_id, restream_client_secret)| {
                    let client_id = restream_client_id.to_owned();
                    let client_secret = restream_client_secret.to_owned();
                    Restream::new(client_id, client_secret)
                }),
        }
    }

    pub async fn apply_channel_settings(
        &mut self,
        hidden: &mut Hidden,
        settings: &ChannelSettings,
    ) -> Result<()> {
        if let Some(peer_tube) = &mut self.peer_tube {
            peer_tube.apply(settings).await?
        }
        if let Some(((restream, access_token), refresh_token)) = { self.restream.as_ref() }
            .zip(hidden.restream_access_token.as_mut())
            .zip(hidden.restream_refresh_token.as_mut())
        {
            let channel_ids = &hidden.restream_channel_ids;
            restream
                .apply(access_token, refresh_token, channel_ids, settings)
                .await?
        }

        Ok(())
    }
}
