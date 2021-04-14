use super::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// PT 站的域名，如 `ourbits.club`
    domain: String,

    cookies: String,

    /// 签到消息通知人
    email: String,
}

pub struct Signer {
    client: Client,
    domain: String,
    email: String,
}
impl Signer {
    fn header(cookie: &str) -> Result<HeaderMap> {
        let mut header = HeaderMap::new();
        header.append(header::COOKIE, cookie.parse()?);
        Ok(header)
    }

    pub fn regex_match(body: &str) -> Result<String, String> {
        let body = regex::Regex::new(r"<(/?)b>")
            .unwrap()
            .replace_all(&body, "")
            .replace(' ', "");
        let result_regex =
            regex::Regex::new(r"这是您的第\d+次签到，已连续签到\d+天，本次签到获得\d+个魔力值。")
                .unwrap();
        if let Some(cap) = result_regex.captures(&body) {
            info!("签到成功");
            Ok(cap.get(0).unwrap().as_str().to_string())
        } else if body.contains("已经签到过了") {
            Err("已经签到过了".to_string())
        } else {
            warn!("Unknown body: {}", body);
            Err("未知错误，未从 body 中解析出数据。".to_string())
        }
    }
}
#[async_trait]
impl super::Signer for Signer {
    type Config = Config;
    type Outcome = String;

    fn new(config: Self::Config) -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36")
            .default_headers(Self::header(&config.cookies)?)
            .build()?;

        Ok(Self {
            client,
            domain: config.domain.clone(),
            email: config.email.clone(),
        })
    }

    fn name(&self) -> String {
        format!("PT 站 {}", self.domain)
    }

    fn notice_receiver(&self) -> &str {
        &self.email
    }

    fn success_body(&self, outcome: &String) -> String {
        format!("PT {} 签到成功：{}", self.domain, outcome)
    }

    async fn sign(&self) -> Result<String> {
        info!("开始 {} 的签到 (user {})", self.domain, self.email);
        let url = format!("https://{}/attendance.php", self.domain);
        let r = self.client.get(&url).send().await?;
        info!("response status: {}", r.status());
        let body = r.text().await?;
        match Self::regex_match(&body) {
            Ok(s) => Ok(s),
            Err(e) => bail!(e),
        }
    }
}

#[test]
fn test_parse() {
    let s = r#"
domain = "baidu.com"
cookies = ""
email = "sdf@example.com"
    "#;
    toml::from_str::<Config>(s).unwrap();
}

#[test]
fn test_regex() {
    let s = r#"did="outer"align="center"class="outer"style="padding-top:20px;padding-bottom:20px">
    <p>这是您的第<b>233 </b> 次签到，已连续签到<b>1</b>天，本次签到获得<b> 100</b>个魔力值。</p></td></tr></table>
    <ul><li>首次签到获得10个魔力值。</li><li>每次签到可额外获得1个魔力值，直到100封顶。</li><li><ol><li>连续签到10天后，每次签到额外获得5魔力值（不累计）。</li><li>连续签到20天后，每次签到额外获得20魔力值（不累计）。</li><li>连续签到30天后，每次签到额外获得40魔力值（不累计）。</li></ol></li></ul></td></tr></table>"
    "#;
    assert_eq!(
        Signer::regex_match(s),
        Ok("这是您的第233次签到，已连续签到1天，本次签到获得100个魔力值。".to_string())
    );
}
