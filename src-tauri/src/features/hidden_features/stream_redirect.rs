use std::{num::NonZeroU16, time::SystemTime};

use actix_web::{
    get,
    http::header::LOCATION,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::{anyhow, Result};
use getset::Setters;
use serde_json::json;
use tokio::task::JoinHandle;

async fn fetch_channel_id(
    client: &reqwest::Client,
    origin: &str,
    channel_name: &str,
) -> Result<String> {
    let id = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .subsec_micros();
    let json = client
        .post(format!("{}/api/1", origin))
        .header("X-Requested-With", "XMLHttpRequest")
        .json(&json! ({
            "jsonrpc": "2.0",
            "id": id,
            "method": "getChannels"
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let channels = json
        .get("result")
        .and_then(|x| x.as_array())
        .ok_or_else(|| anyhow!("invalid json"))?;
    let channel = channels
        .iter()
        .find(|&x| {
            x.get("info")
                .and_then(|x| x.as_object())
                .and_then(|x| x.get("name"))
                .and_then(|x| x.as_str())
                .map(|x| x == channel_name)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("channel not found"))?;
    Ok(channel
        .get("channelId")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("invalid channel json"))?
        .to_owned())
}

#[get("/stream/my.flv")]
async fn get_my_flv(data: web::Data<(NonZeroU16, String)>) -> impl Responder {
    let (port, channel_name) = data.get_ref();
    let origin = format!("http://127.0.0.1:{}", port);
    let client = reqwest::Client::new();
    let channel_id = if let Ok(channel_id) =
        fetch_channel_id(&client, &origin, &format!("{channel_name} (IPv6)")).await
    {
        channel_id
    } else if let Ok(channel_id) = fetch_channel_id(&client, &origin, channel_name).await {
        channel_id
    } else {
        return HttpResponse::NotFound().finish();
    };
    let url = format!("{}/stream/{}.flv", origin, channel_id);

    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, url))
        .finish()
}

#[derive(Default, Setters)]
pub struct StreamRedirect {
    join_handle: Option<JoinHandle<Result<()>>>,
}

impl StreamRedirect {
    pub async fn run(
        &mut self,
        listen_port: NonZeroU16,
        communication_port: NonZeroU16,
        channel_name: String,
    ) {
        if let Some(handle) = self.join_handle.take() {
            handle.abort();
            let _ = handle.await;
        };
        self.join_handle = Some(tokio::spawn(async move {
            let factory = move || {
                App::new()
                    .app_data(Data::new((communication_port, channel_name.clone())))
                    .service(get_my_flv)
            };
            let server = HttpServer::new(factory)
                .bind(("127.0.0.1", listen_port.get()))?
                .bind(("::1", listen_port.get()))?
                .run();
            server.await?;
            Ok(())
        }));
    }
}
