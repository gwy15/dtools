use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use std::{path::PathBuf, time::SystemTime};
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub output: PathBuf,
    pub subscriptions: Vec<Subscription>,
    pub receiver: String,
}

#[derive(Debug, Deserialize)]
pub struct Subscription {
    pub name: String,
    pub cache: PathBuf,
    pub url: String,
    #[serde(default)]
    pub replacements: Vec<String>,
    #[serde(default)]
    pub expire: Option<DateTime<chrono::FixedOffset>>,
}
impl Subscription {
    pub async fn get(&self) -> Result<String> {
        if let Some(t) = self.expire.as_ref() {
            if &Utc::now() > t {
                bail!("The subscription has expired, datetime = {}.", t);
            }
        }

        if let Some(parent) = self.cache.parent() {
            fs::create_dir_all(parent).await?;
        }

        let cache_hit = self.cache.exists()
            && (SystemTime::now().duration_since(fs::metadata(&self.cache).await?.modified()?)?
                < std::time::Duration::from_secs(3600));

        let content = if cache_hit {
            debug!("cache hit, use file cache {:?}", self.cache);
            fs::read_to_string(&self.cache).await?
        } else {
            debug!("downloading url for {}", self.name);
            let r = request::get(&self.url).await?;
            if r.status() != request::StatusCode::OK {
                bail!("Status code = {}", r.status())
            }
            debug!("download success.");
            let content = r.text().await?;
            fs::write(&self.cache, &content).await?;
            content
        };
        Ok(content)
    }
}
