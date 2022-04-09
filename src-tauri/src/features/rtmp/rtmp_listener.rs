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
    port: Option<NonZeroU16>,
    listener_handle: Option<JoinHandle<()>>,
}

impl RtmpListener {
    pub fn new() -> Self {
        Self {
            delegate: None,
            port: None,
            listener_handle: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn RtmpListenerDelegate + Send + Sync>) {
        self.delegate = Some(delegate);
    }

    pub fn port(&self) -> Option<NonZeroU16> {
        self.port
    }

    pub fn stop_listener(&mut self) {
        if let Some(listener_handle) = &self.listener_handle {
            listener_handle.abort();
            self.listener_handle = None;
            self.port = None;
        }
    }

    pub async fn spawn_listener(&mut self, rtmp_listen_port: NonZeroU16) -> anyhow::Result<()> {
        assert!(self.listener_handle.is_none());
        let delegate = self.delegate.clone().unwrap();
        let rtmp_listen_host = format!("0.0.0.0:{}", rtmp_listen_port);
        let listener = TcpListener::bind(&rtmp_listen_host).await?;
        self.port = Some(rtmp_listen_port);
        self.listener_handle = Some(spawn(async move {
            debug!("listening on {}", rtmp_listen_port);

            let delegate = delegate.clone();
            loop {
                let (incoming, _addr) = listener.accept().await.unwrap();

                log::trace!("on_connect begin");
                delegate.upgrade().unwrap().on_connect(incoming).await;
                log::trace!("on_connect end");
            }
        }));
        Ok(())
    }
}
