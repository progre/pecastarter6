mod logger_core;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::core::{
    entities::settings::{ChannelSettings, GeneralSettings, Settings},
    utils::failure::Failure,
};

use self::logger_core::Logger;

type BoxedOnError = Box<dyn Send + Sync + Fn(Failure)>;

pub struct LoggerController {
    logger: Mutex<Option<Logger>>,
    on_error: Arc<std::sync::Mutex<Option<BoxedOnError>>>,
}

impl LoggerController {
    pub fn new() -> Self {
        Self {
            logger: Mutex::new(None),
            on_error: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn set_on_error(&self, on_error: BoxedOnError) {
        *self.on_error.lock().unwrap() = Some(on_error);
    }

    fn spawn_logger(
        &self,
        ipv4_channel_id: Option<String>,
        ipv6_channel_id: Option<String>,
        settings: &Settings,
    ) -> Logger {
        let on_error = self.on_error.clone();
        Logger::spawn(
            &settings.other_settings.log_output_directory,
            ipv4_channel_id,
            ipv6_channel_id,
            &settings.general_settings.channel_name[0],
            settings.general_settings.peer_cast_port,
            Box::new(move |err| {
                if let Some(on_error) = on_error.lock().unwrap().as_ref() {
                    on_error(err);
                }
            }),
        )
    }

    pub async fn on_broadcast(
        &self,
        ipv4_channel_id: Option<String>,
        ipv6_channel_id: Option<String>,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        let log_output_directory = &settings.other_settings.log_output_directory;
        if settings.other_settings.log_enabled && !log_output_directory.is_empty() {
            let logger = self.spawn_logger(ipv4_channel_id, ipv6_channel_id, settings);
            let channel = &settings.channel_settings;
            logger
                .put_info(&channel.genre[0], &channel.desc[0], &channel.comment[0])
                .await?;
            *self.logger.lock().await = Some(logger);
        };
        Ok(())
    }

    pub async fn on_change_general_settings(&self, general_settings: &GeneralSettings) {
        if let Some(logger) = self.logger.lock().await.as_mut() {
            logger.set_peer_cast_port(general_settings.peer_cast_port);
        }
    }

    pub async fn on_change_channel_settings(
        &self,
        channel: &ChannelSettings,
    ) -> anyhow::Result<()> {
        if let Some(logger) = self.logger.lock().await.as_mut() {
            logger
                .put_info(&channel.genre[0], &channel.desc[0], &channel.comment[0])
                .await?;
        }
        Ok(())
    }

    pub async fn on_change_other_settings(
        &self,
        ipv4_channel_id: Option<String>,
        ipv6_channel_id: Option<String>,
        settings: &Settings,
        broadcasting: bool,
    ) -> anyhow::Result<()> {
        let mut logger_opt = self.logger.lock().await;
        if !settings.other_settings.log_enabled || !broadcasting {
            // Here is a blank line for disable clippy
            if let Some(logger) = logger_opt.as_mut() {
                logger.abort();
                *logger_opt = None;
            }
            return Ok(());
        }
        if logger_opt.is_none() {
            let logger = self.spawn_logger(ipv4_channel_id, ipv6_channel_id, settings);
            *logger_opt = Some(logger);
            let channel = &settings.channel_settings;
            let logger = logger_opt.as_ref().unwrap();
            logger
                .put_info(&channel.genre[0], &channel.desc[0], &channel.comment[0])
                .await?;
        }
        Ok(())
    }

    pub async fn on_stop_channel(&self) -> anyhow::Result<()> {
        let mut logger_opt = self.logger.lock().await;
        if let Some(logger) = logger_opt.as_mut() {
            logger.put_info("", "", "（配信終了）").await?;
            logger.abort();
            *logger_opt = None;
        }
        Ok(())
    }
}
