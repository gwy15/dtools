use super::prelude::*;
use utils;

#[derive(Debug, Deserialize)]
pub struct Config {
    cookies: String,
    email: String,
    proxy: Option<String>,
}

pub struct Signer {
    client: Client,
    email: String,
}
impl Signer {
    fn url(path: &str) -> String {
        format!("https://www.v2ex.com{}", path)
    }
}

#[async_trait]
impl super::Signer for Signer {
    type Config = Config;
    type Outcome = String;

    fn new(config: Config) -> Result<Self> {
        let mut client_builder = utils::client_builder().default_headers(header! {
            header::COOKIE => &config.cookies,
            header::REFERER => Self::url("/mission/daily"),
        });
        if let Some(proxy) = config.proxy {
            client_builder = client_builder.proxy(request::Proxy::all(proxy)?);
        }

        let client = client_builder.build()?;
        Ok(Self {
            client,
            email: config.email,
        })
    }

    fn name(&self) -> String {
        "v2ex".to_string()
    }

    fn notice_receiver(&self) -> &str {
        &self.email
    }

    async fn sign(&self) -> Result<String> {
        let resp = self.client.get(Self::url("/mission/daily")).send().await?;
        let text = resp.text().await?;
        trace!("/mission/daily response text: {}", text);
        // find redeem
        let redeem_regex = Regex::new(r"/mission/daily/redeem\?once=\d+")?;
        let result = redeem_regex
            .captures(&text)
            .and_then(|cap| cap.get(0))
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("Failed to find redeem once token."))?;
        let resp = self.client.get(Self::url(result)).send().await?;
        debug!("redeem response: {:?}", resp.status());
        let response_text = resp.text().await?;
        trace!("response text: {}", response_text);

        let reward_regex = Regex::new(r"已连续登录 \d+ 天")?;
        let reward_text = reward_regex
            .captures(&response_text)
            .and_then(|cap| cap.get(0))
            .map(|m| m.as_str())
            .ok_or_else(|| anyhow!("没找到提示信息"))?;

        Ok(format!("签到成功，{}", reward_text))
    }
}
