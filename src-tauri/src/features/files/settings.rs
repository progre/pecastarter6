use std::{io::ErrorKind, path::Path, process::exit};

use log::error;
use tokio::fs::{OpenOptions, create_dir, read_to_string, rename, write};

use crate::core::{
    entities::settings::{Settings, StoredSettings, StoringSettings},
    utils::{
        dialog::{show_confirm, show_dialog},
        tcp::find_free_port,
    },
};

async fn rename_bak(app_dir: &Path, base_path: &str) {
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

    if let Err(err) = rename(app_dir.join("settings.json"), path).await {
        log::error!("err {}", err);
    }
}

pub async fn load_settings_and_show_dialog_if_error(app_dir: &Path) -> Settings {
    let old_path = app_dir.parent().unwrap().join("PeCa Starter");
    if old_path.exists() && !app_dir.exists() {
        if !show_confirm(&format!(
            "設定ファイルを移動します。\nfrom: {}\nto: {}",
            old_path.to_string_lossy(),
            app_dir.to_string_lossy()
        )) {
            exit(0);
            // -> !
        }
        if let Err(err) = rename(old_path, &app_dir).await {
            show_dialog(&format!("ファイルの移動に失敗しました。({:?})", err));
            panic!("{:?}", err);
            // -> !
        }
    }
    let path = app_dir.join("settings.json");
    match read_to_string(&path).await {
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                error!("{:?}", err);
                show_dialog(&format!(
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
                show_dialog(&format!(
                    "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                    err
                ));
                rename_bak(app_dir, &path.to_string_lossy()).await;
                Settings::default()
            }
            Ok(settings) => {
                log::trace!("{:?}", settings);
                settings.into_internal()
            }
        },
    }
}

pub async fn save_settings_and_show_dialog_if_error(app_dir: &Path, settings: &Settings) {
    if let Err(err) = create_dir(app_dir).await
        && err.kind() != ErrorKind::AlreadyExists
    {
        panic!("{:?}", err);
    }
    let opt = write(
        app_dir.join("settings.json"),
        serde_json::to_string_pretty(&StoringSettings::from(settings)).unwrap(),
    )
    .await;
    if let Err(err) = opt {
        error!("{:?}", err);
        show_dialog(&format!("設定ファイルの保存に失敗しました。({:?})", err));
    }
}
