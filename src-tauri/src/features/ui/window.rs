use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use serde_json::{json, Value};
use tauri::{generate_context, generate_handler, Manager, PageLoadPayload};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    core::entities::{
        settings::{ChannelSettings, GeneralSettings, Settings, YellowPagesSettings},
        yp_config::YPConfig,
    },
    features::terms_check,
};

/*
 * # tokio スレッドと std スレッドの間の通信について
 *
 * tokio スレッド -> std スレッド
 * * スレッドセーフ?の AppHandle に一任
 * * channel でのデータ転送
 *
 * std スレッド -> tokio スレッド
 * * delegate (但し即時処理のみ対応)
 * * channel でのデータ転送 (但し戻り値を持てない)
 */

#[async_trait]
pub trait WindowDelegate {
    fn on_load_page(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

struct WindowState {
    delegate: Weak<DynSendSyncWindowDelegate>,
}

trait StateExt {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate>;
}

impl StateExt for tauri::State<'_, WindowState> {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate> {
        self.delegate.upgrade().unwrap()
    }
}

#[tauri::command]
async fn fetch_hash(url: String) -> Result<String, String> {
    terms_check::fetch_hash(&url)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn initial_data(
    state: tauri::State<'_, WindowState>,
) -> Result<(Vec<YPConfig>, Settings), ()> {
    Ok(state.delegate().initial_data().await)
}

#[tauri::command]
async fn set_general_settings(
    general_settings: GeneralSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_general_settings(general_settings)
        .await;

    // WTF: 戻り値Resultが必須。バグ？
    Ok(())
}

#[tauri::command]
async fn set_yellow_pages_settings(
    yellow_pages_settings: YellowPagesSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_yellow_pages_settings(yellow_pages_settings)
        .await;

    Ok(())
}

#[tauri::command]
async fn set_channel_settings(
    channel_settings: ChannelSettings,
    state: tauri::State<'_, WindowState>,
) -> Result<(), ()> {
    state
        .delegate()
        .on_change_channel_settings(channel_settings)
        .await;

    Ok(())
}

fn on_page_load(window: tauri::Window, _page_load_payload: PageLoadPayload) {
    (window.state() as tauri::State<'_, WindowState>)
        .delegate()
        .on_load_page();
}

fn run_tauri(mut command_rx: mpsc::Receiver<Command>, delegate: Weak<DynSendSyncWindowDelegate>) {
    let app = tauri::Builder::default()
        .manage(WindowState { delegate })
        .invoke_handler(generate_handler![
            fetch_hash,
            initial_data,
            set_general_settings,
            set_yellow_pages_settings,
            set_channel_settings,
        ])
        .on_page_load(on_page_load)
        .any_thread()
        .build(generate_context!())
        .expect("error while running tauri application");
    app.run(move |app_handle, _| {
        if let Ok(command) = command_rx.try_recv() {
            match command {
                Command::Emit(event, payload) => {
                    app_handle.emit_all(event, payload).unwrap();
                }
                Command::UpdateTitle(title_status) => app_handle
                    .get_window("main")
                    .unwrap()
                    .set_title(&format!(
                        "{} {}",
                        app_handle.package_info().name,
                        title_status,
                    ))
                    .unwrap(),
            }
        }
    });
}

enum Command {
    Emit(&'static str, Value),
    UpdateTitle(String),
}

pub struct Window {
    command_tx: mpsc::Sender<Command>,
    command_rx: Mutex<Option<mpsc::Receiver<Command>>>,
}

impl Window {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(16);
        Self {
            command_tx,
            command_rx: Mutex::new(Some(command_rx)),
        }
    }

    pub fn run(&self, delegate: Weak<DynSendSyncWindowDelegate>) -> JoinHandle<()> {
        let command_rx = self.command_rx.lock().unwrap().take().unwrap();
        let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            run_tauri(command_rx, delegate);
            oneshot_tx.send(()).unwrap();
        });
        tokio::spawn(async {
            oneshot_rx.await.unwrap();
        })
    }

    pub async fn push_settings(&self, settings: Settings) {
        self.command_tx
            .send(Command::Emit(
                "push_settings",
                serde_json::to_value(settings).unwrap(),
            ))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub async fn notify(&self, level: &str, message: &str) {
        self.command_tx
            .send(Command::Emit(
                "notify",
                json!({
                    "level": level,
                    "message": message
                }),
            ))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub async fn set_rtmp(&self, rtmp: &str) {
        self.command_tx
            .send(Command::Emit("status", json!({ "rtmp": rtmp })))
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }

    pub fn set_title_status(&self, title_status: String) {
        self.command_tx
            // TODO: 高負荷時にクラッシュする可能性がある
            .try_send(Command::UpdateTitle(title_status))
            .unwrap_or_else(|err| panic!("{}", err));
    }
}
