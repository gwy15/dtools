use super::prelude::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    domain: String,
    email: String,
    // 需要登录
    username: String,
    password: String,
    #[serde(default)]
    proxy: Option<String>,
    #[serde(default)]
    force_success_msg: Option<String>,
}

pub struct Signer {
    client: Client,
    config: Config,
}
impl Signer {
    pub fn url(&self, path: &str) -> String {
        format!("https://{}{}", self.config.domain, path)
    }
}
#[async_trait]
impl super::Signer for Signer {
    type Config = Config;
    type Outcome = String;
    fn new(config: Self::Config) -> Result<Self> {
        let mut builder = utils::client_builder();
        if let Some(proxy) = config.proxy.clone() {
            builder = builder.proxy(request::Proxy::all(proxy)?);
        }
        let client = builder.build()?;
        Ok(Self { client, config })
    }

    fn name(&self) -> String {
        format!("机场 {}", self.config.domain)
    }

    fn notice_receiver(&self) -> &str {
        &self.config.email
    }

    async fn sign(&self) -> Result<String> {
        let resp = self
            .client
            .post(self.url("/auth/login"))
            .json(&json!({
                "email": self.config.username,
                "passwd": self.config.password,
                "code": "",
                "remember_me": "week"
            }))
            .send()
            .await?;
        debug!("login response: {:?}", resp.status());

        // 签到接口
        let resp = self
            .client
            .post(self.url("/user/checkin"))
            .headers(header! {
                header::REFERER => self.url("/user"),
            })
            .send()
            .await?;
        debug!("sign in response: {:?}", resp.status());

        let text = resp.text().await?;
        debug!("sign in response text: {}", text);
        let data: SignResponse = serde_json::from_str(&text)?;

        if data.success {
            if let Some(traffic) = data.traffic {
                let msg = format!(
                    "{}，今天已使用 {}，剩余流量 {}",
                    data.msg, traffic.today_used, traffic.unused
                );

                return Ok(msg);
            }
        }
        if let Some(force_success) = &self.config.force_success_msg {
            if data.msg.contains(force_success.as_str()) {
                let msg = format!("{} 强制成功", self.config.domain);
                return Ok(msg);
            }
        }
        warn!("{}", data.msg);
        return Err(anyhow!("签到失败：{}", data.msg));
    }

    fn success_body(&self, outcome: &String) -> String {
        outcome.clone()
    }
}

#[derive(Debug, Deserialize)]
struct TrafficInfo {
    #[serde(rename = "todayUsedTraffic")]
    today_used: String,
    #[serde(rename = "unUsedTraffic")]
    unused: String,
}

#[derive(Debug, Deserialize)]
struct SignResponse {
    #[serde(rename = "ret", deserialize_with = "utils::bool_from_int")]
    success: bool,

    msg: String,

    #[serde(rename = "trafficInfo")]
    traffic: Option<TrafficInfo>,
}

#[test]
fn test_parse() {
    let s = r#"{"ret":0,"msg":"\u60a8\u4f3c\u4e4e\u5df2\u7ecf\u7b7e\u5230\u8fc7\u4e86..."}"#;
    let r: SignResponse = serde_json::from_str(s).unwrap();
    assert_eq!(r.success, false);
    assert_eq!(r.msg, "您似乎已经签到过了...");
    assert!(r.traffic.is_none());

    let s = r#"{"msg":"\u83b7\u5f97\u4e86 92MB \u6d41\u91cf.","unflowtraffic":1088,"traffic":"0.6GB","trafficInfo":{"todayUsedTraffic":"0.29GB","lastUsedTraffic":"0B","unUsedTraffic":"91GB"},"ret":1}"#;
    let r: SignResponse = serde_json::from_str(s).unwrap();
    assert_eq!(r.success, true);
    assert_eq!(r.msg, "获得了 92MB 流量.");
    assert_eq!(r.traffic.unwrap().unused, "91GB");
}
