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
use std::{path::PathBuf, process::Command, str::FromStr};

use clap::Parser;
use features::{
    terms_check,
    ui::window::{InvokeMessageExt, WindowDelegate, WindowState},
};
use tauri::{
    Manager, generate_context,
    ipc::{Invoke, InvokeBody},
};

use crate::core::args::Args;

fn invoke_handler(
    Invoke {
        message,
        resolver,
        acl: _,
    }: Invoke,
) -> bool {
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
                let InvokeBody::Json(payload) = message.payload() else {
                    unreachable!();
                };
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
                let app_config_dir = message
                    .webview_ref()
                    .app_handle()
                    .path()
                    .app_config_dir()
                    .unwrap();
                Command::new(cmd)
                    .arg(app_config_dir.to_str().unwrap())
                    .output()
                    .unwrap();
            }
            _ => panic!("unknown command"),
        }
    });
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if cfg!(debug_assertions) {
        unsafe { std::env::set_var("RUST_LOG", "app=trace,reqwest=trace") };
        env_logger::init();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|tauri_app| {
            let path_resolver = tauri_app.path();
            let app_config_dir = path_resolver.app_config_dir().unwrap();
            let resource_dir = path_resolver.resource_dir().unwrap();

            let settings_path = Args::parse()
                .settings_path
                .map(|x| PathBuf::from_str(&x).unwrap())
                .unwrap_or_else(|| app_config_dir.join("settings.json"));

            let app = tauri::async_runtime::block_on(async {
                App::new(&app_config_dir, &resource_dir, &settings_path).await
            });
            let weak = app.ui.ui_window_delegate_weak();
            *app.ui.window().app_handle().lock().unwrap() = Some(tauri_app.handle().to_owned());
            tauri_app.manage(WindowState::new(app, weak.clone()));
            weak.upgrade().unwrap().on_build_app();

            Ok(())
        })
        .invoke_handler(invoke_handler)
        .run(generate_context!())
        .map_err(|err| {
            const NOTE: &str =
                "WebView2 ランタイムをインストールするとこのエラーが解決する可能性があります。";
            let mut note = "";
            if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(err)) = &err
                && err.to_string().contains("WebView2")
            {
                note = NOTE;
            }
            show_dialog(&format!(
                "アプリケーションの起動に失敗しました。{}({}) ",
                note, err
            ));
            err
        })
        .expect("error while running tauri application");
}
