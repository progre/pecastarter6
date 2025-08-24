use std::{num::NonZero, path::Path, sync::Arc, time::Duration};

use tokio::{sync::Mutex, time::sleep};
use versions::Version;

use crate::{
    core::{
        app::App,
        entities::{settings::Settings, yp_config::YPConfig},
        utils::failure::Failure,
    },
    features::{
        files::settings::save_settings_and_show_dialog_if_error,
        hidden_features::jpnkn_bbs_auto_comment::JpnknBbsAutoComment, logger::LoggerController,
        peercast::broadcasting::Broadcasting,
    },
};

fn jpnkn_bbs_auto_comment(settings: &Settings, app: Arc<App>) -> Option<JpnknBbsAutoComment> {
    if settings
        .other_settings
        .hidden
        .as_ref()
        .is_some_and(|x| x.jpnkn_bbs_auto_comment)
    {
        Some(JpnknBbsAutoComment::new(app))
    } else {
        None
    }
}

async fn start_channel(
    broadcasting: &mut Broadcasting,
    yp_configs: &[YPConfig],
    settings: &Settings,
    logger_controller: &LoggerController,
    jpnkn_bbs_auto_comment: Option<&mut JpnknBbsAutoComment>,
) -> Result<NonZero<u16>, Failure> {
    let rtmp_conn_port = broadcasting.broadcast(yp_configs, settings).await?;

    let ipv4_id = broadcasting.ipv4_id().clone();
    let ipv6_id = broadcasting.ipv6_id().clone();
    logger_controller
        .on_broadcast(ipv4_id, ipv6_id, settings)
        .await
        .map_err(|err| Failure::Warn(err.to_string()))?;
    if let Some(jpnkn_bbs_auto_comment) = jpnkn_bbs_auto_comment {
        jpnkn_bbs_auto_comment.on_broadcast().await;
    }

    Ok(rtmp_conn_port)
}

pub async fn start_broadcast(
    app: &Arc<App>,
    app_dir: &Path,
) -> Result<(NonZero<u16>, Option<JpnknBbsAutoComment>), Failure> {
    let (rtmp_conn_port, jpnkn_bbs_auto_comment) = {
        let settings = app.settings.lock().await;
        let mut broadcasting = app.broadcasting.lock().await;
        let mut jpnkn_bbs_auto_comment = jpnkn_bbs_auto_comment(&settings, app.clone());
        let rtmp_conn_port = start_channel(
            &mut broadcasting,
            &app.yp_configs,
            &settings,
            &app.logger_controller,
            jpnkn_bbs_auto_comment.as_mut(),
        )
        .await?;
        (rtmp_conn_port, jpnkn_bbs_auto_comment)
    };

    {
        let mut settings = app.settings.lock().await;
        app.update_histories(&mut settings, &app.ui);
        save_settings_and_show_dialog_if_error(app_dir, &settings).await;
    }

    app.ui.set_rtmp("streaming".to_owned());

    Ok((rtmp_conn_port, jpnkn_bbs_auto_comment))
}

async fn stop_channel(
    broadcasting: &Mutex<Broadcasting>,
    peer_cast_port: NonZero<u16>,
    logger_controller: &LoggerController,
    jpnkn_bbs_auto_comment: Option<&mut JpnknBbsAutoComment>,
) -> Result<(), Failure> {
    logger_controller
        .on_stop_channel()
        .await
        .map_err(|err| Failure::Warn(err.to_string()))?;
    if let Some(jpnkn_bbs_auto_comment) = jpnkn_bbs_auto_comment {
        jpnkn_bbs_auto_comment.on_stop_channel();
    }

    broadcasting.lock().await.stop(peer_cast_port).await
}

pub async fn stop_broadcast(
    app: &App,
    jpnkn_bbs_auto_comment: Option<&mut JpnknBbsAutoComment>,
) -> Result<(), Failure> {
    let pecast_version = {
        let settings = app.settings.lock().await;
        app.broadcasting
            .lock()
            .await
            .fetch_version(settings.general_settings.peer_cast_port)
            .await?
    };
    if pecast_version < Version::new("3.1.0.0").unwrap() {
        // NOTE: If the channel is deleted within 3 seconds of the stream closed,
        //       a tcp listener on PeerCastStation will remain.
        //       https://github.com/kumaryu/peercaststation/issues/490
        sleep(Duration::from_secs(6)).await;
    }

    app.ui.set_rtmp("listening".to_owned());

    {
        let settings = app.settings.lock().await;
        stop_channel(
            &app.broadcasting,
            settings.general_settings.peer_cast_port,
            &app.logger_controller,
            jpnkn_bbs_auto_comment,
        )
        .await?;
    }
    Ok(())
}
