use std::io::ErrorKind;

use log::error;
use tokio::fs::{create_dir, read_to_string, rename, write, OpenOptions};

use crate::{
    core::{
        entities::settings::{Settings, StoredSettings, StoringSettings},
        utils::tcp::find_free_port,
    },
    features::files::dialog::show_file_error_dialog,
};

use super::APP_DIR;

async fn rename_bak(base_path: &str) {
    let mut i = 0;
    let path = loop {
        let idx = if i == 0 { "".into() } else { format!(".{}", i) };
        let path = format!("{}{}.bak", base_path, idx);
        log::trace!("{:?}", path);
        if OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .is_ok()
        {
            break path;
        }
        i += 1;
    };

    if let Err(err) = rename(APP_DIR.join("settings.json"), path).await {
        log::error!("err {}", err);
    }
}

pub async fn load_settings_and_show_dialog_if_error() -> Settings {
    let path = APP_DIR.join("settings.json");
    match read_to_string(&path).await {
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
        Ok(str) => match deser_hjson::from_str::<StoredSettings>(&str) {
            Err(err) => {
                error!("{:?}", err);
                show_file_error_dialog(&format!(
                    "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                    err
                ));
                rename_bak(&path.to_string_lossy()).await;
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
