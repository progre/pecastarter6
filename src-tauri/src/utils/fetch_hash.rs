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
