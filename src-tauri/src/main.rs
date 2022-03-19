#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::App;

mod app;
mod entities;
mod failure;
mod features;
mod utils;
mod window;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();

    App::run().await;
}
