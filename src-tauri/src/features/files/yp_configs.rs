use std::{
    collections::BTreeMap,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::error;
use tokio::fs::{read_dir, read_to_string};

use crate::core::{entities::yp_config::YPConfig, utils::dialog::show_dialog};

async fn read_yp_config(path: PathBuf) -> Result<YPConfig> {
    let json_src = read_to_string(path).await?;
    Ok(serde_json::from_str::<YPConfig>(&json_src)?)
}

async fn read_yp_config_and_show_dialog_if_error(path: PathBuf) -> Option<YPConfig> {
    match read_yp_config(path).await {
        Ok(yp_configs) => Some(yp_configs),
        Err(err) => {
            error!("{:?}", err);
            show_dialog(&format!(
                "YP設定ファイルの読み込みに失敗しました。({:?})",
                err
            ));
            None
        }
    }
}

pub async fn read_yp_configs_and_show_dialog_if_error(
    app_dir: &Path,
    resource_dir: &Path,
) -> Vec<YPConfig> {
    let exe_dir_yp = resource_dir.join("yp");
    let app_dir_yp = app_dir.join("yp");

    let mut yp_configs = BTreeMap::new();

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
            }
            if let Some(config) = read_yp_config_and_show_dialog_if_error(entry.path()).await {
                yp_configs.insert(file_name, config);
            }
        }
    }

    yp_configs.into_values().collect()
}
