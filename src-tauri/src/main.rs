#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;
mod features;

use core::{
    app::App,
    utils::{dialog::show_dialog, tcp::find_free_port},
};
use std::{path::PathBuf, process::Command};

use features::{
    terms_check,
    ui::window::{InvokeMessageExt, WindowDelegate, WindowState},
};
use tauri::{
    api::path::{app_config_dir, resource_dir},
    generate_context, Env, Invoke, Manager,
};

fn invoke_handler(Invoke { message, resolver }: Invoke, app_dir: PathBuf) {
    tauri::async_runtime::spawn(async move {
        let delegate = message.state_ref().get::<WindowState>().delegate();
        match message.command() {
            "initial_data" => {
                resolver.resolve(delegate.initial_data().await);
            }
            "put_settings" => {
                if let Some(settings) = message.get_from_payload("generalSettings") {
                    delegate.on_change_general_settings(settings).await;
                }
                if let Some(settings) = message.get_from_payload("yellowPagesSettings") {
                    delegate.on_change_yellow_pages_settings(settings).await;
                }
                if let Some(settings) = message.get_from_payload("channelSettings") {
                    delegate.on_change_channel_settings(settings).await;
                }
                if let Some(settings) = message.get_from_payload("otherSettings") {
                    delegate.on_change_other_settings(settings).await;
                }
            }
            "fetch_hash" => {
                let payload = message.payload();
                let url = payload.get("url").unwrap().as_str().unwrap();
                let selector = payload.get("selector").and_then(|x| x.as_str());
                resolver.resolve(terms_check::fetch_hash(url, selector).await.unwrap());
            }
            "find_free_port" => {
                resolver.resolve(find_free_port().await.unwrap());
            }
            "open_app_dir" => {
                let cmd = if cfg!(target_os = "macos") {
                    "open"
                } else {
                    "explorer.exe"
                };
                Command::new(cmd)
                    .arg(app_dir.to_str().unwrap())
                    .output()
                    .unwrap();
            }
            _ => panic!("unknown command"),
        }
    });
}

fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "app=trace,reqwest=trace");
        env_logger::init();
    }

    let context = generate_context!();
    let app_dir = app_config_dir(context.config()).unwrap();
    let resource_dir = resource_dir(context.package_info(), &Env::default()).unwrap();

    let app = tauri::async_runtime::block_on(async { App::new(&app_dir, &resource_dir).await });
    let weak = app.ui.ui_window_delegate_weak();

    let tauri_app = tauri::Builder::default()
        .manage(WindowState::new(weak.clone()))
        .invoke_handler(move |invoke| invoke_handler(invoke, app_dir.clone()))
        .build(context)
        .map_err(|err| {
            const NOTE: &str =
                "WebView2 ランタイムをインストールするとこのエラーが解決する可能性があります。";
            let mut note = "";
            if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(err)) = &err {
                if err.to_string().contains("WebView2") {
                    note = NOTE;
                }
            }
            show_dialog(&format!(
                "アプリケーションの起動に失敗しました。{}({}) ",
                note, err
            ));
            err
        })
        .expect("error while running tauri application");

    *app.ui.window().app_handle().lock().unwrap() = Some(tauri_app.app_handle());
    weak.upgrade().unwrap().on_build_app();
    tauri_app.run(|_, _| {});
}
