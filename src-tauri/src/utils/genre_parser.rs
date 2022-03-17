use crate::entities::{settings::EachYellowPagesSettings, yp_config::YPConfig};

fn supported(yp_config: &YPConfig, feature: &str) -> bool {
    yp_config.supported_params.iter().any(|x| x == feature)
}

pub fn stringify(genre: &str, yp_config: &YPConfig, settings: &EachYellowPagesSettings) -> String {
    format!(
        "{}{}{}{}{}{}{}",
        yp_config.prefix_header,
        if supported(yp_config, "namespace") && !settings.namespace.is_empty() {
            format!("{}:", &settings.namespace)
        } else {
            "".to_owned()
        },
        if supported(yp_config, "hide_listeners") && settings.hide_listeners {
            "?"
        } else {
            ""
        },
        if supported(yp_config, "port_bandwidth_check") {
            (0..settings.port_bandwidth_check)
                .map(|_| "@")
                .collect::<Vec<_>>()
                .join("")
        } else {
            "".to_owned()
        },
        if supported(yp_config, "no_log") && settings.no_log {
            "+"
        } else {
            ""
        },
        genre,
        if supported(yp_config, "icon") {
            &settings.icon
        } else {
            ""
        },
    )
}
