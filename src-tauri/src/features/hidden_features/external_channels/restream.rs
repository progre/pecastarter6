// https://api.restream.io/login?response_type=code&client_id=$CLIENT_ID&redirect_uri=https://localhost&state=undefined
// curl -X POST https://api.restream.io/oauth/token -H "Content-Type: application/x-www-form-urlencoded" --user "$CLIENT_ID:$CLIENT_SECRET" --data "grant_type=authorization_code&redirect_uri=https://localhost&code=$CODE"
// curl https://api.restream.io/v2/user/channel/all -H "Authorization: Bearer $ACCESS_TOKEN"

use std::future::Future;

use anyhow::Result;
use reqwest::{Response, StatusCode};
use serde_json::json;

use crate::core::entities::settings::ChannelSettings;

#[derive(Default, serde::Deserialize)]
struct OauthClient {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Default, serde::Deserialize)]
struct OauthToken {
    // token_type: String,
    access_token: String,
    refresh_token: String,
    // expires_in: i32,
}

async fn oauth_token_from_refresh_token(
    client: &reqwest::Client,
    oauth_client: &OauthClient,
    refresh_token: &str,
) -> reqwest::Result<OauthToken> {
    let req = client
        .post("https://api.restream.io/oauth/token")
        .basic_auth(&oauth_client.client_id, Some(&oauth_client.client_secret))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ]);
    let resp = req.send().await?;
    resp.error_for_status()?.json().await
}

pub struct Restream {
    client: reqwest::Client,
    oauth_client: OauthClient,
}

impl Restream {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Default::default(),
            oauth_client: OauthClient {
                client_id,
                client_secret,
            },
        }
    }

    async fn auto_refresh<T>(
        &self,
        oauth_token: &mut OauthToken,
        callback: impl Fn(&reqwest::Client, &str) -> T,
    ) -> reqwest::Result<Response>
    where
        T: Future<Output = reqwest::Result<Response>>,
    {
        let resp = callback(&self.client, &oauth_token.access_token).await?;
        if resp.status() != StatusCode::UNAUTHORIZED {
            return Ok(resp);
        }

        let oauth_client = &self.oauth_client;
        let refresh_token = &oauth_token.refresh_token;
        *oauth_token =
            oauth_token_from_refresh_token(&self.client, oauth_client, refresh_token).await?;

        callback(&self.client, &oauth_token.access_token).await
    }

    pub async fn apply(
        &self,
        access_token: &mut String,
        refresh_token: &mut String,
        channel_ids: &[u64],
        settings: &ChannelSettings,
    ) -> Result<()> {
        let mut oauth_token = OauthToken {
            access_token: access_token.to_owned(),
            refresh_token: refresh_token.to_owned(),
        };
        for channel_id in channel_ids {
            self.auto_refresh(&mut oauth_token, move |client, access_token| {
                let url = format!(
                    "https://api.restream.io/v2/user/channel-meta/{}",
                    channel_id
                );
                let json = json!({ "title": settings.desc });
                let req = client.patch(url).bearer_auth(access_token).json(&json);
                req.send()
            })
            .await?
            .error_for_status()?;
        }

        *access_token = oauth_token.access_token;
        *refresh_token = oauth_token.refresh_token;

        Ok(())
    }
}
