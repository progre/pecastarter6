use std::path::PathBuf;

use once_cell::sync::Lazy;
use tauri::{api::path, generate_context};

pub mod settings;
pub mod yp_configs;

static APP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let context = generate_context!();
    path::app_dir(context.config()).unwrap()
});