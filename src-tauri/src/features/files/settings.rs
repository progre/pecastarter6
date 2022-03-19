use std::io::ErrorKind;

use log::error;
use tokio::fs::{create_dir, read_to_string, write};

use crate::core::entities::settings::Settings;

use super::APP_DIR;

pub async fn load_settings_and_show_dialog_if_error(dialog: fn(&str)) -> Settings {
    match read_to_string(APP_DIR.join("settings.json")).await {
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                error!("{:?}", err);
                dialog(&format!(
                    "設定ファイルの読み込みに失敗しました。({:?})",
                    err
                ));
            }
            Settings::default()
        }
        Ok(str) => match serde_json::from_str::<Settings>(&str) {
            Err(err) => {
                error!("{:?}", err);
                dialog(&format!(
                    "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                    err
                ));
                Settings::default()
            }
            Ok(settings) => {
                log::trace!("{:?}", settings);
                settings
            }
        },
    }
}

pub async fn save_settings_and_show_dialog_if_error(settings: &Settings, dialog: fn(&str)) {
    if let Err(err) = create_dir(APP_DIR.as_path()).await {
        if err.kind() != ErrorKind::AlreadyExists {
            panic!("{:?}", err);
        }
    }
    let opt = write(
        APP_DIR.join("settings.json"),
        serde_json::to_string_pretty(settings).unwrap(),
    )
    .await;
    if let Err(err) = opt {
        error!("{:?}", err);
        dialog(&format!("設定ファイルの保存に失敗しました。({:?})", err));
    }
}
