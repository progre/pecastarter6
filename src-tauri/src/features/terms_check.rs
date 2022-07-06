use crate::core::entities::{settings::Settings, yp_config::YPConfig};

use nipper::Document;
use sha2::{Digest, Sha256};

pub async fn fetch_hash(url: &str, selector: Option<&str>) -> anyhow::Result<String> {
    let res = reqwest::get(url).await?;

    let mut hasher = Sha256::new();
    let html = res.text().await?;
    if let Some(selector) = selector {
        let part = Document::from(&html).select(selector).html();
        log::trace!("{}", part);
        let src = part.as_bytes();
        hasher.update(src);
    } else {
        let src = html.as_bytes();
        hasher.update(src);
    };
    let result = hasher.finalize();
    let hash = result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");

    Ok(hash)
}

async fn expired_yp_terms<'a, 'b>(
    yp_configs: &'a [YPConfig],
    settings: &'b Settings,
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
        .map(|yp_config| {
            (
                &yp_config.terms_url as &str,
                yp_config.terms_selector.as_ref().map(|x| x as &str),
            )
        })
        .collect::<Vec<_>>();

    let mut terms_hashes = Vec::new();
    for (yp_terms_url, yp_terms_selector) in yp_terms_urls {
        let hash = fetch_hash(yp_terms_url, yp_terms_selector).await?;
        terms_hashes.push((yp_terms_url, hash));
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
