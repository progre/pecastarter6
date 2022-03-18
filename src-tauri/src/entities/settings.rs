use std::{collections::HashMap, io::ErrorKind, num::NonZeroU16, path::PathBuf};

use log::error;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::{
    api::{dialog, path},
    generate_context,
};
use tokio::fs::{create_dir, read_to_string, write};

static APP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let context = generate_context!();
    path::app_dir(context.config()).unwrap()
});

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub peer_cast_port: NonZeroU16,
    pub channel_name: Vec<String>,
    pub rtmp_listen_port: NonZeroU16,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        GeneralSettings {
            peer_cast_port: NonZeroU16::new(7144u16).unwrap(),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSettings {
    pub genre: Vec<String>,
    pub desc: Vec<String>,
    pub comment: Vec<String>,
    pub contact_url: Vec<String>,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        ChannelSettings {
            genre: vec!["".to_owned()],
            desc: vec!["".to_owned()],
            comment: vec!["".to_owned()],
            contact_url: vec!["".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub general_settings: GeneralSettings,
    pub yellow_pages_settings: YellowPagesSettings,
    pub channel_settings: ChannelSettings,
}

unsafe impl Send for Settings {}
unsafe impl Sync for Settings {}

impl Settings {
    pub async fn load() -> Self {
        match read_to_string(APP_DIR.join("settings.json")).await {
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    error!("{:?}", err);
                    let none: Option<&tauri::Window> = None;
                    dialog::blocking::message(
                        none,
                        "Fatal",
                        format!("設定ファイルの読み込みに失敗しました。({:?})", err),
                    );
                }
                Settings::default()
            }
            Ok(str) => match serde_json::from_str::<Settings>(&str) {
                Err(err) => {
                    error!("{:?}", err);
                    let none: Option<&tauri::Window> = None;
                    dialog::blocking::message(
                        none,
                        "Fatal",
                        format!(
                            "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                            err
                        ),
                    );
                    Settings::default()
                }
                Ok(settings) => {
                    log::trace!("{:?}", settings);
                    settings
                }
            },
        }
    }

    pub async fn save(&self) {
        if let Err(err) = create_dir(APP_DIR.as_path()).await {
            if err.kind() != ErrorKind::AlreadyExists {
                panic!("{:?}", err);
            }
        }
        let opt = write(
            APP_DIR.join("settings.json"),
            serde_json::to_string_pretty(self).unwrap(),
        )
        .await;
        if let Err(err) = opt {
            error!("{:?}", err);
            let none: Option<&tauri::Window> = None;
            dialog::blocking::message(
                none,
                "Fatal",
                format!("設定ファイルの保存に失敗しました。({:?})", err),
            );
        }
    }
}
