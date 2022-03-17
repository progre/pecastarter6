use std::{collections::HashMap, env::current_exe, io::ErrorKind, path::PathBuf};

use anyhow::Result;
use log::error;
use once_cell::sync::Lazy;
use tauri::{
    api::{dialog, path},
    generate_context,
};
use tokio::fs::{read_dir, read_to_string};

use crate::entities::yp_config::YPConfig;

static APP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let context = generate_context!();
    path::app_dir(context.config()).unwrap()
});

async fn read_yp_config(path: PathBuf) -> Result<YPConfig> {
    let json_src = read_to_string(path).await?;
    Ok(serde_json::from_str::<YPConfig>(&json_src)?)
}

async fn read_yp_config_and_show_dialog_if_error(path: PathBuf) -> Option<YPConfig> {
    match read_yp_config(path).await {
        Ok(yp_configs) => Some(yp_configs),
        Err(err) => {
            error!("{:?}", err);
            let none: Option<&tauri::Window> = None;
            dialog::blocking::message(
                none,
                "Fatal",
                format!("YP設定ファイルの読み込みに失敗しました。({:?})", err),
            );
            None
        }
    }
}

pub async fn read_yp_configs_and_show_dialog_if_error() -> Vec<YPConfig> {
    let exe_dir_yp = current_exe().unwrap().with_file_name("yp");
    let app_dir_yp = APP_DIR.join("yp");

    let mut yp_configs = HashMap::new();

    for dir in [app_dir_yp, exe_dir_yp] {
        log::trace!("{:?}", dir);
        let mut iter = match read_dir(dir).await {
            Ok(iter) => iter,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    continue;
                }
                panic!();
            }
        };
        while let Some(entry) = iter.next_entry().await.unwrap() {
            let file_name = entry.file_name();
            if yp_configs.contains_key(&file_name)
                || !file_name.to_string_lossy().ends_with(".json")
                || !entry.file_type().await.unwrap().is_file()
            {
                continue;
            };
            if let Some(config) = read_yp_config_and_show_dialog_if_error(entry.path()).await {
                yp_configs.insert(file_name, config);
            }
        }
    }

    yp_configs.into_values().collect()
}
