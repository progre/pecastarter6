use std::sync::Weak;

use crate::core::entities::{settings::Settings, yp_config::YPConfig};

use super::{rtmp_listener::RtmpListener, RtmpListenerDelegate};

pub struct RtmpServer {
    rtmp_listener: RtmpListener,
}

impl RtmpServer {
    pub fn new() -> Self {
        Self {
            rtmp_listener: RtmpListener::new(),
        }
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn RtmpListenerDelegate + Send + Sync>) {
        self.rtmp_listener.set_delegate(delegate);
    }

    pub fn listen_rtmp_if_need(&mut self, yp_configs: &[YPConfig], settings: &Settings) -> bool {
        let hosts = [
            &settings.yellow_pages_settings.ipv4.host,
            &settings.yellow_pages_settings.ipv6.host,
        ];
        let has_yp = hosts.iter().any(|host| !host.is_empty());
        let agreed_all_terms = hosts
            .into_iter()
            .map(|host| yp_configs.iter().find(|config| &config.host == host))
            .flatten()
            .map(|config| &config.terms_url)
            .all(|terms_url| {
                settings
                    .yellow_pages_settings
                    .agreed_terms
                    .contains_key(terms_url)
            });

        let should_listen = has_yp && agreed_all_terms;
        if should_listen {
            let equals_running_port =
                self.rtmp_listener.port() == Some(settings.general_settings.rtmp_listen_port);
            if equals_running_port {
                return true;
            }
            self.rtmp_listener.stop_listener();
            self.rtmp_listener
                .spawn_listener(settings.general_settings.rtmp_listen_port);
            true
        } else {
            let running = self.rtmp_listener.port().is_some();
            if !running {
                return false;
            }
            self.rtmp_listener.stop_listener();
            false
        }
    }
}
