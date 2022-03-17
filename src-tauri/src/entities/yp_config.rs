use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YPConfig {
    pub name: String,
    #[serde(rename = "termsURL")]
    pub terms_url: String,
    pub host: String,
    pub support_ipv6: bool,
    pub prefix_header: String,
    pub supported_params: Vec<String>,
}
