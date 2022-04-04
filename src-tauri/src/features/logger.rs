use chrono::{DateTime, Local, SecondsFormat};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    spawn,
    sync::Mutex,
    task::JoinHandle,
    time::{interval, Duration},
};

use crate::core::entities::settings::{ChannelSettings, Settings};

fn to_csv_column(column: &str) -> String {
    column.replace('"', "\"\"")
}

fn to_csv_line(
    local: DateTime<Local>,
    listeners: Option<i32>,
    relays: Option<i32>,
    genre: &str,
    description: &str,
    comment: &str,
) -> String {
    format!(
        "{},{},{},{},{},{}",
        local.to_rfc3339_opts(SecondsFormat::Secs, true),
        listeners.map(|x| x.to_string()).unwrap_or_default(),
        relays.map(|x| x.to_string()).unwrap_or_default(),
        to_csv_column(genre),
        to_csv_column(description),
        to_csv_column(comment)
    )
}

pub struct Logger {
    join_handle: JoinHandle<()>,
    path: String,
    on_error: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl Logger {
    pub fn spawn(directory: &str, channel_name: &str) -> Self {
        let join_handle = spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            interval.tick().await;
            loop {
                interval.tick().await;
                // TODO: write listeners and relays
            }
        });
        Self {
            join_handle,
            path: format!(
                "{}/{}_{}.csv",
                directory,
                Local::now().format("%Y%m%d%H%M%S"),
                channel_name
            ),
            on_error: None,
        }
    }

    pub async fn put_info(&self, genre: &str, desc: &str, comment: &str) -> anyhow::Result<()> {
        let local = Local::now();
        let line = to_csv_line(local, None, None, genre, desc, comment);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write(line.as_bytes()).await?;

        Ok(())
    }

    pub fn abort(&mut self) {
        self.join_handle.abort();
    }
}

pub struct LoggerController {
    logger: Mutex<Option<Logger>>,
}

impl LoggerController {
    pub fn new() -> Self {
        Self {
            logger: Mutex::new(None),
        }
    }

    pub async fn on_broadcast(&self, settings: &Settings) -> anyhow::Result<()> {
        let log_output_directory = &settings.other_settings.log_output_directory;
        if settings.other_settings.log_enabled && !log_output_directory.is_empty() {
            let logger = Logger::spawn(
                log_output_directory,
                &settings.general_settings.channel_name[0],
            );
            let channel = &settings.channel_settings;
            logger
                .put_info(&channel.genre[0], &channel.desc[0], &channel.comment[0])
                .await?;
            *self.logger.lock().await = Some(logger);
        };
        Ok(())
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
            let logger = Logger::spawn(
                &settings.other_settings.log_output_directory,
                &settings.general_settings.channel_name[0],
            );
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
