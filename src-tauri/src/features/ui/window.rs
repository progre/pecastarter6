use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tauri::{generate_context, Invoke, InvokeMessage, Manager, PageLoadPayload};
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
 * * tauri::async_runtime::spawn でラップ
 */

#[async_trait]
pub trait WindowDelegate {
    async fn on_load_page(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

struct WindowState {
    delegate: Weak<DynSendSyncWindowDelegate>,
}

impl WindowState {
    fn delegate(&self) -> Arc<DynSendSyncWindowDelegate> {
        self.delegate.upgrade().unwrap()
    }
}

enum Command {
    Emit(&'static str, Value),
    UpdateTitle(String),
}

#[async_trait]
trait InvokeMessageExt {
    fn get_from_payload<T>(&self, param: &str) -> T
    where
        T: DeserializeOwned;
}

#[async_trait]
impl InvokeMessageExt for InvokeMessage {
    fn get_from_payload<T>(&self, param: &str) -> T
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(self.payload().get(param).unwrap().clone()).unwrap()
    }
}

fn run_tauri(delegate: Weak<DynSendSyncWindowDelegate>, mut command_rx: mpsc::Receiver<Command>) {
    tauri::Builder::default()
        .manage(WindowState { delegate })
        .invoke_handler(|Invoke { message, resolver }| {
            tauri::async_runtime::spawn(async move {
                let delegate = message.state_ref().get::<WindowState>().delegate();
                match message.command() {
                    "fetch_hash" => {
                        resolver.resolve(
                            terms_check::fetch_hash(message.payload().as_str().unwrap())
                                .await
                                .unwrap(),
                        );
                    }
                    "initial_data" => {
                        resolver.resolve(delegate.initial_data().await);
                    }
                    "set_general_settings" => {
                        let settings = message.get_from_payload("generalSettings");
                        delegate.on_change_general_settings(settings).await;
                    }
                    "set_yellow_pages_settings" => {
                        let settings = message.get_from_payload("yellowPagesSettings");
                        delegate.on_change_yellow_pages_settings(settings).await;
                    }
                    "set_channel_settings" => {
                        let settings = message.get_from_payload("channelSettings");
                        delegate.on_change_channel_settings(settings).await;
                    }
                    _ => panic!("unknown command"),
                }
            });
        })
        .on_page_load(|window: tauri::Window, _: PageLoadPayload| {
            tauri::async_runtime::spawn(async move {
                let delegate = (window.state() as tauri::State<'_, WindowState>).delegate();
                delegate.on_load_page().await;
            });
        })
        .any_thread()
        .build(generate_context!())
        .expect("error while running tauri application")
        .run(move |app_handle, _| {
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
            run_tauri(delegate, command_rx);
            oneshot_tx.send(()).unwrap();
        });
        tokio::spawn(async {
            oneshot_rx.await.unwrap();
        })
    }

    pub async fn push_settings(&self, settings: Settings) {
        self.send(Command::Emit(
            "push_settings",
            serde_json::to_value(settings).unwrap(),
        ))
        .await;
    }

    pub async fn notify(&self, level: &str, message: &str) {
        self.send(Command::Emit(
            "notify",
            json!({
                "level": level,
                "message": message
            }),
        ))
        .await;
    }

    pub async fn set_rtmp(&self, rtmp: &str) {
        self.send(Command::Emit("status", json!({ "rtmp": rtmp })))
            .await;
    }

    pub async fn set_title_status(&self, title_status: String) {
        self.send(Command::UpdateTitle(title_status)).await;
    }

    async fn send(&self, command: Command) {
        self.command_tx
            .send(command)
            .await
            .unwrap_or_else(|err| panic!("{}", err));
    }
}
