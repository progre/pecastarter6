use std::io::ErrorKind;

use log::error;
use tokio::fs::{create_dir, read_to_string, write};

use crate::{
    core::{
        entities::settings::{Settings, StoredSettings, StoringSettings},
        utils::tcp::find_free_port,
    },
    features::files::dialog::show_file_error_dialog,
};

use super::APP_DIR;

pub async fn load_settings_and_show_dialog_if_error() -> Settings {
    match read_to_string(APP_DIR.join("settings.json")).await {
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                error!("{:?}", err);
                show_file_error_dialog(&format!(
                    "設定ファイルの読み込みに失敗しました。({:?})",
                    err
                ));
            }
            let mut default = Settings::default();
            default.general_settings.peer_cast_rtmp_port = find_free_port().await.unwrap().into();
            default
        }
        Ok(str) => match serde_json::from_str::<StoredSettings>(&str) {
            Err(err) => {
                error!("{:?}", err);
                show_file_error_dialog(&format!(
                    "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                    err
                ));
                Settings::default()
            }
            Ok(settings) => {
                log::trace!("{:?}", settings);
                settings.into_internal()
            }
        },
    }
}

pub async fn save_settings_and_show_dialog_if_error(settings: &Settings) {
    if let Err(err) = create_dir(APP_DIR.as_path()).await {
        if err.kind() != ErrorKind::AlreadyExists {
            panic!("{:?}", err);
        }
    }
    let opt = write(
        APP_DIR.join("settings.json"),
        serde_json::to_string_pretty(&StoringSettings::from(settings)).unwrap(),
    )
    .await;
    if let Err(err) = opt {
        error!("{:?}", err);
        show_file_error_dialog(&format!("設定ファイルの保存に失敗しました。({:?})", err));
    }
}
