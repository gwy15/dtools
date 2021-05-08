use std::time::SystemTime;
use uuid::Uuid;

use super::prelude::*;

lazy_static::lazy_static! {
    static ref APP_VERSION: &'static str = "2.3.0";
    static ref UA: String =
        format!("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0_1 like Mac OS X) \
        AppleWebKit/605.1.15 (KHTML, like Gecko) miHoYoBBS/{}", *APP_VERSION);
    static ref INDEX_URL: &'static str = "https://webstatic.mihoyo.com/bbs/event/signin-ys/index.html";
    static ref HEADERS: HeaderMap = {
        let mut hm = HeaderMap::new();
        hm.insert(header::USER_AGENT, HeaderValue::from_str(&UA).unwrap());
        hm.insert(header::REFERER, HeaderValue::from_static(&INDEX_URL));
        hm.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
        hm.insert("x-rpc-app_version", HeaderValue::from_static(&APP_VERSION));
        hm
    };
}

#[derive(Debug, Deserialize)]
struct Character {
    pub game_biz: String,
    pub game_uid: String,
    pub nickname: String,
    pub region_name: String,
}

/// 配置
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub cookies: String,
    pub email: String,
}

/// 逻辑
pub struct Signer {
    client: Client,
    uuid: String,
    notice_receiver: String,
}
impl Signer {
    async fn get_uids(&self) -> Result<Vec<Character>> {
        static URL: &str = "https://api-takumi.mihoyo.com/binding/api/getUserGameRolesByCookie";
        let r = self
            .client
            .get(URL)
            .query(&[("game_biz", "hk4e_cn")])
            .send()
            .await?;
        debug!("get uid: response received： {:?}", r);

        #[derive(Debug, Deserialize)]
        struct Response {
            pub retcode: i64,
            pub message: String,
            pub data: Option<Data>,
        }
        #[derive(Debug, Deserialize)]
        struct Data {
            pub list: Vec<Character>,
        }

        let response: Response = r.json().await?;
        debug!("response: {:?}", response);
        if response.retcode != 0 || response.data.is_none() {
            return Err(anyhow::anyhow!("请求用户角色返回：{}", response.message));
        }
        // safe here
        let chars = response.data.unwrap().list;
        debug!("characters: {:?}", chars);
        Ok(chars)
    }

    fn generate_ds() -> String {
        static SALT: &str = "h8w582wxwgqvahcdkpvdhbh2w9casgfl";
        static RANDOM: &str = "114514";
        let t: u64 = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let sign = format!(
            "{:x}",
            md5::compute(format!(
                "salt={salt}&t={t}&r={r}",
                salt = SALT,
                t = t,
                r = RANDOM
            ))
        );
        format!("{},{},{}", t, RANDOM, sign)
    }

    async fn sign_character(&self, char: Character) -> Result<()> {
        static URL: &str = "https://api-takumi.mihoyo.com/event/bbs_sign_reward/sign";
        info!("准备签到角色 {}", char.nickname);

        let response = self
            .client
            .post(URL)
            .json(&json!({
                "act_id": "e202009291139501",
                "region": "cn_gf01",
                "uid": char.game_uid
            }))
            .header("x-rpc-device_id", HeaderValue::from_str(&self.uuid)?)
            .header("x-rpc-client_type", HeaderValue::from_static("5"))
            .header("DS", HeaderValue::from_str(&Self::generate_ds())?)
            .send()
            .await?;

        let text = response.text().await?;
        debug!("response: {:?}", text);

        #[derive(Debug, Deserialize)]
        struct Response {
            pub retcode: i64,
            pub message: String,
        }
        let response: Response = serde_json::from_str(&text)?;
        if response.retcode != 0 {
            error!("签到错误：{:?}", response);
            return Err(anyhow::anyhow!(
                "角色 {} 签到错误：{}",
                char.nickname,
                response.message
            ));
        }
        info!("角色 {} 签到成功：{}", char.nickname, response.message);
        Ok(())
    }
}

#[async_trait]
impl super::Signer for Signer {
    type Config = Config;
    type Outcome = ();

    fn name(&self) -> String {
        "原神".to_string()
    }

    fn new(config: Config) -> Result<Self> {
        let cookies = &config.cookies;
        let mut headers = HEADERS.clone();
        headers.insert(header::COOKIE, HeaderValue::from_str(cookies)?);

        let client = Client::builder().default_headers(headers).build()?;
        debug!("client built.");

        let uuid = Uuid::new_v3(&uuid::Uuid::NAMESPACE_URL, cookies.as_bytes())
            .to_simple()
            .encode_upper(&mut Uuid::encode_buffer())
            .to_string();

        Ok(Self {
            client,
            uuid,
            notice_receiver: config.email,
        })
    }

    fn notice_receiver(&self) -> &str {
        self.notice_receiver.as_str()
    }

    async fn sign(&self) -> Result<()> {
        let uids = self.get_uids().await?;
        for uid in uids {
            self.sign_character(uid).await?;
        }
        Ok(())
    }
}
