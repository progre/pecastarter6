use std::sync::{Mutex, Weak};

use async_trait::async_trait;
use futures::future::BoxFuture;
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
    async fn on_load_page(&self);
    async fn initial_data(&self) -> (Vec<YPConfig>, Settings);
    async fn on_change_general_settings(&self, general_settings: GeneralSettings);
    async fn on_change_yellow_pages_settings(&self, yellow_pages_settings: YellowPagesSettings);
    async fn on_change_channel_settings(&self, channel_settings: ChannelSettings);
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

type InitialData = Box<dyn Send + Sync + Fn() -> BoxFuture<'static, (Vec<YPConfig>, Settings)>>;

struct WindowState {
    event_tx: mpsc::Sender<Event>,
    initial_data: InitialData,
}

impl WindowState {
    fn blocking_send(&self, event: Event) {
        self.event_tx
            .blocking_send(event)
            .unwrap_or_else(|err| panic!("{}", err));
    }
}

enum Command {
    Emit(&'static str, Value),
    UpdateTitle(String),
}

enum Event {
    PageLoad,
    ChangeGeneralSettings(GeneralSettings),
    ChangeYellowPagesSettings(YellowPagesSettings),
    ChangeChannelSettings(ChannelSettings),
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
    Ok((state.initial_data)().await)
}

#[tauri::command]
fn set_general_settings(state: tauri::State<'_, WindowState>, general_settings: GeneralSettings) {
    state.blocking_send(Event::ChangeGeneralSettings(general_settings));
}

#[tauri::command]
fn set_yellow_pages_settings(
    state: tauri::State<'_, WindowState>,
    yellow_pages_settings: YellowPagesSettings,
) {
    state.blocking_send(Event::ChangeYellowPagesSettings(yellow_pages_settings));
}

#[tauri::command]
fn set_channel_settings(state: tauri::State<'_, WindowState>, channel_settings: ChannelSettings) {
    state.blocking_send(Event::ChangeChannelSettings(channel_settings));
}

fn on_page_load(window: tauri::Window, _page_load_payload: PageLoadPayload) {
    (window.state() as tauri::State<'_, WindowState>).blocking_send(Event::PageLoad);
}

fn run_tauri(
    mut command_rx: mpsc::Receiver<Command>,
    event_tx: mpsc::Sender<Event>,
    initial_data_closure: InitialData,
) {
    let app = tauri::Builder::default()
        .manage(WindowState {
            event_tx,
            initial_data: initial_data_closure,
        })
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
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
        let std_thread_delegate = delegate.clone();
        std::thread::spawn(move || {
            run_tauri(
                command_rx,
                event_tx,
                Box::new(move || {
                    let delegate = std_thread_delegate.clone();
                    Box::pin(async move { delegate.upgrade().unwrap().initial_data().await })
                }),
            );
            oneshot_tx.send(()).unwrap();
        });
        tokio::spawn(async move {
            loop {
                let event = event_rx.recv().await.unwrap();
                let delegate = delegate.upgrade().unwrap();
                match event {
                    Event::PageLoad => delegate.on_load_page().await,
                    Event::ChangeGeneralSettings(general_settings) => {
                        delegate.on_change_general_settings(general_settings).await
                    }
                    Event::ChangeYellowPagesSettings(yellow_pages_settings) => {
                        delegate
                            .on_change_yellow_pages_settings(yellow_pages_settings)
                            .await
                    }
                    Event::ChangeChannelSettings(channel_settings) => {
                        delegate.on_change_channel_settings(channel_settings).await
                    }
                }
            }
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
