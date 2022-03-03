use std::num::NonZeroU16;
use std::sync::Weak;

use async_trait::async_trait;
use log::debug;
use tokio::net::TcpStream;
use tokio::{net::TcpListener, spawn, task::JoinHandle};

#[async_trait]
pub trait RtmpListenerDelegate {
    async fn on_connect(&self, incoming: TcpStream);
}

pub struct RtmpListener {
    delegate: Option<Weak<dyn RtmpListenerDelegate + Send + Sync>>,
    listener_handle: Option<JoinHandle<()>>,
}

impl RtmpListener {
    pub fn new() -> Self {
        Self {
            delegate: None,
            listener_handle: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn RtmpListenerDelegate + Send + Sync>) {
        self.delegate = Some(delegate);
    }

    pub fn spawn_listener(&mut self, rtmp_listen_port: NonZeroU16) {
        if let Some(listener_handle) = &self.listener_handle {
            listener_handle.abort();
            self.listener_handle = None;
        }
        let delegate = self.delegate.clone().unwrap();
        self.listener_handle = Some(spawn(async move {
            let rtmp_listen_host = format!("0.0.0.0:{}", rtmp_listen_port);
            let listener = TcpListener::bind(&rtmp_listen_host).await.unwrap();
            debug!("listening on {}", rtmp_listen_port);

            let delegate = delegate.clone();
            loop {
                let (incoming, _addr) = listener.accept().await.unwrap();

                delegate.upgrade().unwrap().on_connect(incoming).await;
            }
        }));
    }
}
