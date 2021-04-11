pub mod genshin;

mod prelude {
    pub use anyhow::{Error, Result};
    pub use async_trait::async_trait;
    pub use reqwest::{
        header::{self, HeaderMap, HeaderValue},
        Client,
    };
    pub use serde::Deserialize;
    pub use serde_json::json;
}

use prelude::{Error, Result};

#[async_trait::async_trait]
pub trait Signer: Sized {
    type Config;

    /// 签到器名字，如 “原神”
    fn name(&self) -> String;

    /// 签到结果通知（邮件地址）
    fn notice_receiver(&self) -> &str;

    fn new(config: Self::Config) -> Result<Self>;

    async fn sign(&self) -> Result<()>;

    fn success_msg(&self) -> String {
        format!("{} 签到成功", self.name())
    }

    fn success_body(&self) -> String {
        format!("{} 签到成功啦", self.name())
    }

    fn fail_msg(&self, _e: &Error) -> String {
        format!("【签到失败】{} 签到失败，请手动补签！", self.name())
    }

    fn fail_body(&self, e: &Error) -> String {
        format!("失败原因：{:?}", e)
    }
}
