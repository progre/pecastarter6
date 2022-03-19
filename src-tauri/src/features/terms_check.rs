use crate::entities::{settings::Settings, yp_config::YPConfig};

use sha2::{Digest, Sha256};

pub async fn fetch_hash(url: &str) -> anyhow::Result<String> {
    let res = reqwest::get(url).await?;

    let mut hasher = Sha256::new();
    hasher.update(res.bytes().await?);
    let result = hasher.finalize();
    let hash = result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");

    Ok(hash)
}

async fn expired_yp_terms<'a>(
    yp_configs: &'a [YPConfig],
    settings: &Settings,
) -> anyhow::Result<Vec<&'a str>> {
    let hosts = [
        &settings.yellow_pages_settings.ipv4.host,
        &settings.yellow_pages_settings.ipv6.host,
    ];
    let yp_terms_urls = hosts
        .into_iter()
        .filter(|host| !host.is_empty())
        .map(|host| yp_configs.iter().find(|x| &x.host == host).unwrap())
        .filter(|yp_config| !yp_config.ignore_terms_check)
        .map(|yp_config| &yp_config.terms_url as &str)
        .collect::<Vec<_>>();
    let mut terms_hashes = Vec::new();
    for yp_terms_url in yp_terms_urls {
        terms_hashes.push((yp_terms_url, fetch_hash(yp_terms_url).await?));
    }
    let updated_terms = terms_hashes
        .into_iter()
        .filter(|(url, hash)| settings.yellow_pages_settings.agreed_terms.get(*url) != Some(hash))
        .map(|(url, _)| url)
        .collect::<Vec<_>>();
    Ok(updated_terms)
}

pub async fn check_expired_terms<'a>(
    yp_configs: &[YPConfig],
    settings: &mut Settings,
) -> anyhow::Result<bool> {
    let expired_yp_terms = expired_yp_terms(yp_configs, settings).await?;
    if expired_yp_terms.is_empty() {
        return Ok(true);
    }
    for url in expired_yp_terms {
        settings.yellow_pages_settings.agreed_terms.remove(url);
    }

    Ok(false)
}
