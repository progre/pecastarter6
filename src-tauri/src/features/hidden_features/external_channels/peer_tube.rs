use std::future::Future;

use anyhow::Result;
use regex::Regex;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::core::entities::settings::ChannelSettings;

const FEDIMOVIE_BASE_URL: &str = "https://fedimovie.com/api/v1";

fn genre_to_category(genre: &str) -> i32 {
    if Regex::new(r"(?i)ゲーム|game").unwrap().is_match(genre) {
        7 // Gaming
    } else if genre.contains("外配信") {
        6 // Travels
    } else {
        15 // Science & Technology
    }
}

async fn oauth_client_local(client: &reqwest::Client) -> reqwest::Result<OauthClientLocal> {
    client
        .get(format!("{}/oauth-clients/local", FEDIMOVIE_BASE_URL))
        .send()
        .await?
        .json()
        .await
}

async fn token_from_password(
    client: &reqwest::Client,
    oauth_client_local: &OauthClientLocal,
    username: &str,
    password: &str,
) -> reqwest::Result<Token> {
    client
        .post(format!("{}/users/token", FEDIMOVIE_BASE_URL))
        .form(&json!({
            "client_id": oauth_client_local.client_id,
            "client_secret": oauth_client_local.client_secret,
            "grant_type": "password",
            "username": username,
            "password": password,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn token_from_refresh_token(
    client: &reqwest::Client,
    oauth_client_local: &OauthClientLocal,
    refresh_token: &str,
) -> reqwest::Result<Token> {
    client
        .post(format!("{}/users/token", FEDIMOVIE_BASE_URL))
        .form(&json!({
            "client_id": oauth_client_local.client_id,
            "client_secret": oauth_client_local.client_secret,
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn auto_refresh<'a, T, F, G>(
    client: &reqwest::Client,
    fedimovie_oauth_client_local: &mut OauthClientLocal,
    fedimovie_token: &mut Token,
    fedimovie_email: &str,
    fedimovie_password: &str,
    callback: F,
) -> reqwest::Result<T>
where
    F: Fn(reqwest::Client, String) -> G,
    G: Future<Output = reqwest::Result<T>>,
{
    if !fedimovie_oauth_client_local.client_id.is_empty() {
        let resp = Box::pin(callback(
            client.clone(),
            fedimovie_token.access_token.to_owned(),
        ))
        .await;
        match resp {
            Ok(ok) => return Ok(ok),
            Err(err) => {
                if err.status() != Some(StatusCode::UNAUTHORIZED) {
                    return Err(err);
                }
            }
        }

        let resp = token_from_refresh_token(
            client,
            fedimovie_oauth_client_local,
            &fedimovie_token.refresh_token,
        )
        .await;
        match resp {
            Ok(ok) => {
                *fedimovie_token = ok;
                return callback(client.clone(), fedimovie_token.access_token.to_owned()).await;
            }
            Err(err) => {
                if err.status() != Some(StatusCode::UNAUTHORIZED) {
                    return Err(err);
                }
            }
        };
    } else {
        *fedimovie_oauth_client_local = oauth_client_local(client).await?;
    }

    *fedimovie_token = token_from_password(
        client,
        fedimovie_oauth_client_local,
        fedimovie_email,
        fedimovie_password,
    )
    .await?;
    callback(client.clone(), fedimovie_token.access_token.to_owned()).await
}

#[derive(Default, Deserialize)]
pub struct OauthClientLocal {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Default, Deserialize)]
struct Token {
    // token_type: String,
    access_token: String,
    refresh_token: String,
    // expires_in: i32,
    // refresh_token_expires_in: i32,
}

pub struct PeerTube {
    client: reqwest::Client,
    email: String,
    password: String,
    oauth_client_local: OauthClientLocal,
    token: Token,
}

impl PeerTube {
    pub fn new(email: String, password: String) -> Self {
        Self {
            client: Default::default(),
            email,
            password,
            oauth_client_local: Default::default(),
            token: Default::default(),
        }
    }

    async fn auto_refresh<'a, T, F, G>(&mut self, callback: F) -> reqwest::Result<T>
    where
        F: Fn(reqwest::Client, String) -> G,
        G: Future<Output = reqwest::Result<T>>,
    {
        auto_refresh(
            &self.client,
            &mut self.oauth_client_local,
            &mut self.token,
            &self.email,
            &self.password,
            callback,
        )
        .await
    }

    pub async fn apply(&mut self, settings: &ChannelSettings) -> Result<()> {
        #[derive(Debug, Deserialize)]
        struct VideosDatumPrivacy {
            id: u64,
            // label: String,
        }
        #[derive(Debug, Deserialize)]
        struct VideosDatum {
            id: u64,
            privacy: VideosDatumPrivacy,
        }
        #[derive(Debug, Deserialize)]
        struct Videos {
            data: Vec<VideosDatum>,
        }
        let videos = self
            .auto_refresh(|client, access_token| async move {
                client
                    .get(format!(
                        "{}/users/me/videos?isLive=true&skipCount=true",
                        FEDIMOVIE_BASE_URL
                    ))
                    .bearer_auth(&access_token)
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Videos>()
                    .await
            })
            .await?;
        let id = videos
            .data
            .iter()
            .find(|datum| datum.privacy.id == 1)
            .unwrap()
            .id;
        self.auto_refresh(|client, access_token| async move {
            client
                .put(format!("{}/videos/{}", FEDIMOVIE_BASE_URL, id))
                .bearer_auth(&access_token)
                .form(&json!({
                    "category": genre_to_category(&settings.genre),
                    "name": &settings.desc,
                }))
                .send()
                .await?
                .error_for_status()
        })
        .await?;

        Ok(())
    }
}
