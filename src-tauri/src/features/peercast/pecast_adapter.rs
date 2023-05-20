use std::num::{NonZeroU16, NonZeroU32};

use log::{error, trace};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use crate::core::utils::failure::Failure;

use super::view_xml::ViewXml;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub bitrate: Option<NonZeroU32>,
    pub mime_type: &'a str,
    pub genre: &'a str,
    pub desc: &'a str,
    pub comment: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track<'a> {
    pub name: &'a str,
    pub creator: &'a str,
    pub genre: &'a str,
    pub album: &'a str,
    pub url: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Rpc<T>
where
    T: Serialize,
{
    jsonrpc: String,
    id: i32,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

pub struct PeCaStAdapter {
    port: NonZeroU16,
}

impl PeCaStAdapter {
    pub fn new(port: NonZeroU16) -> Self {
        Self { port }
    }

    pub async fn get_version_info(&self) -> Result<String, Failure> {
        let result_json = request_rpc::<()>(self.port, "getVersionInfo", None).await?;
        Ok(result_json
            .as_object()
            .ok_or_else(|| {
                error!("Result is not object.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?
            .get("agentName")
            .ok_or_else(|| {
                error!("agentName is undefined.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?
            .as_str()
            .ok_or_else(|| {
                error!("agentName is not string.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?
            .to_owned())
    }

    pub async fn get_yellow_pages(&self) -> Result<Vec<(i32, String)>, Failure> {
        let result_json = request_rpc::<()>(self.port, "getYellowPages", None).await?;
        let list = result_json.as_array().ok_or_else(|| {
            error!("Result is not array.");
            Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
        })?;
        let list: Vec<_> = list
            .iter()
            .map(|yp| {
                let obj = yp.as_object()?;
                Some((
                    obj.get("yellowPageId")?.as_i64()? as i32,
                    obj.get("uri")?.as_str()?.to_owned(),
                ))
            })
            .collect();
        if list.iter().any(|x| x.is_none()) {
            return Err(Failure::Fatal(
                "Failure communicating with PeerCastStation.".to_owned(),
            ));
        }
        Ok(list.into_iter().map(|x| x.unwrap()).collect())
    }

    pub async fn add_yellow_page(
        &self,
        protocol: &str,
        name: &str,
        announce_uri: &str,
    ) -> Result<i32, Failure> {
        let params = {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Params<'a> {
                protocol: &'a str,
                name: &'a str,
                announce_uri: &'a str,
            }
            Params {
                protocol,
                name,
                announce_uri,
            }
        };
        let result_json = request_rpc(self.port, "addYellowPage", Some(params)).await?;

        (|| result_json.as_object()?.get("yellowPageId")?.as_i64())()
            .map(|x| x as i32)
            .ok_or_else(|| {
                error!("Result is not array.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })
    }

    pub async fn check_ports(&self) -> Result<(), Failure> {
        request_rpc::<()>(self.port, "checkPorts", None)
            .await?
            .as_array()
            .ok_or_else(|| {
                error!("Result is not array.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?;
        Ok(())
    }

    pub async fn get_external_ip_addresses(&self) -> Result<(), Failure> {
        request_rpc::<()>(self.port, "getExternalIPAddresses", None)
            .await?
            .as_array()
            .ok_or_else(|| {
                error!("Result is not array.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn broadcast_channel<'a>(
        &self,
        yellow_page_id: Option<i32>,
        source_uri: &str,
        source_stream: &str,
        content_reader: &str,
        info: &Info<'a>,
        track: &Track<'a>,
        network_type: &str,
    ) -> Result<String, Failure> {
        let params = {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Params<'a> {
                yellow_page_id: Option<i32>,
                source_uri: &'a str,
                source_stream: &'a str,
                content_reader: &'a str,
                info: &'a Info<'a>,
                track: &'a Track<'a>,
                network_type: &'a str,
            }
            Params {
                yellow_page_id,
                source_uri,
                source_stream,
                content_reader,
                info,
                track,
                network_type,
            }
        };
        trace!("broadcast_channel {:?}", params);

        Ok(request_rpc(self.port, "broadcastChannel", Some(params))
            .await?
            .as_str()
            .ok_or_else(|| {
                error!("Result is not string.");
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?
            .to_owned())
    }

    pub async fn set_channel_info<'a>(
        &self,
        channel_id: &'a str,
        info: &'a Info<'a>,
        track: &'a Track<'a>,
    ) -> Result<(), Failure> {
        let params = {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Params<'a> {
                channel_id: &'a str,
                info: &'a Info<'a>,
                track: &'a Track<'a>,
            }
            Params {
                channel_id,
                info,
                track,
            }
        };

        request_rpc(self.port, "setChannelInfo", Some(params))
            .await
            .map(|_| ())
    }

    pub async fn stop_channel(&self, channel_id: &str) -> Result<(), Failure> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Params<'a> {
            channel_id: &'a str,
        }
        let params = Params { channel_id };

        request_rpc(self.port, "stopChannel", Some(params))
            .await
            .map(|_| ())
    }

    pub async fn view_xml(&self) -> Result<ViewXml, Failure> {
        let res = reqwest::get(&format!("http://localhost:{}/admin?cmd=viewxml", self.port))
            .await
            .map_err(|e| {
                error!("{}", e);
                Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
            })?;
        let xml = res.text().await.map_err(|e| {
            error!("{}", e);
            Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
        })?;

        Ok(serde_xml_rs::from_str(&xml).unwrap())
    }
}

async fn request_rpc<T>(
    port: NonZeroU16,
    method: &'static str,
    params: Option<T>,
) -> Result<Value, Failure>
where
    T: Serialize,
{
    let request = Client::new()
        .post(format!("http://localhost:{}/api/1/", port))
        .header("X-Requested-With", "XMLHttpRequest")
        .json(&Rpc {
            jsonrpc: "2.0".to_owned(),
            id: 0,
            method: method.to_owned(),
            params,
        });
    let response = request.send().await.map_err(|e| {
        error!("{}", e);
        Failure::Error(e.to_string())
    })?;

    let payload = response.json::<Value>().await.map_err(|e| {
        error!("{}", e);
        Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
    })?;

    let payload_obj_ref = payload.as_object().ok_or_else(|| {
        error!("Result is not json object.");
        Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
    })?;
    if payload_obj_ref.contains_key("error") {
        log::trace!("{:?}", payload_obj_ref);
        return Err(Failure::Error(
            payload_obj_ref
                .get("error")
                .and_then(|x| x.get("message"))
                .and_then(|x| x.as_str())
                .ok_or_else(|| {
                    error!("Result is not json object.");
                    Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
                })?
                .to_owned(),
        ));
    }
    Ok(payload_obj_ref
        .get("result")
        .ok_or_else(|| {
            error!("Result is not json object.");
            Failure::Fatal("Failure communicating with PeerCastStation.".to_owned())
        })?
        .to_owned())
}
