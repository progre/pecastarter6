use serde::{Deserialize, Serialize};

use crate::core::entities::settings::EachYellowPagesSettings;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YPConfig {
    pub name: String,
    #[serde(rename = "termsURL")]
    pub terms_url: String,
    #[serde(default)]
    pub ignore_terms_check: bool,
    pub host: String,
    pub support_ipv6: bool,
    pub prefix_header: String,
    pub supported_params: Vec<String>,
}

impl YPConfig {
    fn supported(&self, feature: &str) -> bool {
        self.supported_params.iter().any(|x| x == feature)
    }

    pub fn genre_full_text(&self, genre: &str, settings: &EachYellowPagesSettings) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.prefix_header,
            if self.supported("namespace") && !settings.namespace.is_empty() {
                format!("{}:", &settings.namespace)
            } else {
                "".to_owned()
            },
            if self.supported("hide_listeners") && settings.hide_listeners {
                "?"
            } else {
                ""
            },
            if self.supported("port_bandwidth_check") {
                (0..settings.port_bandwidth_check)
                    .map(|_| "@")
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                "".to_owned()
            },
            if self.supported("no_log") && settings.no_log {
                "+"
            } else {
                ""
            },
            genre,
            if self.supported("icon") {
                &settings.icon
            } else {
                ""
            },
        )
    }
}
