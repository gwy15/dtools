use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

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

pub struct Signer {
    client: Client,
}
impl Signer {
    pub fn new(cookies: String) -> Result<Self> {
        let mut headers = HEADERS.clone();
        headers.insert(header::COOKIE, HeaderValue::from_str(&cookies)?);

        let client = Client::builder().default_headers(headers).build()?;

        info!("client built.");

        Ok(Self { client })
    }
    pub async fn sign(self) -> Result<()> {
        let uid = self.get_uid().await?;
        Ok(())
    }

    async fn get_uid(&self) -> Result<i64> {
        static URL: &'static str =
            "https://api-takumi.mihoyo.com/binding/api/getUserGameRolesByCookie";
        let r = self
            .client
            .get(URL)
            .query(&[("game_biz", "hk4e_cn")])
            .send()
            .await?;
        info!("get uid: response received： {:?}", r);

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
        #[derive(Debug, Deserialize)]
        struct Character {
            pub game_biz: String,
            pub game_uid: String,
            pub nickname: String,
            pub region_name: String,
        }

        let response: Response = r.json().await?;
        info!("response: {:?}", response);
        if response.retcode != 0 || response.data.is_none() {
            return Err(anyhow::anyhow!("请求用户角色返回：{}", response.message));
        }
        let chars = response.data.unwrap().list;
        info!("characters: {:?}", chars);
        Ok(1)
    }
}
