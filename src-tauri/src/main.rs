#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;
mod features;

use crate::core::app::App;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "app=trace");
    env_logger::init();

    App::run().await;
}
