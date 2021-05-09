mod prelude {
    pub use super::super::utils;
    pub use anyhow::{Error, Result};
    pub use async_trait::async_trait;
    pub use regex::Regex;
    pub use request::{
        header::{self, HeaderMap, HeaderValue},
        Client,
    };
    pub use serde_json::json;
}

use prelude::{Error, Result};

#[async_trait::async_trait]
pub trait Signer: Sized {
    type Config;
    type Outcome;

    /// 签到器名字，如 “原神”
    fn name(&self) -> String;

    /// 签到结果通知（邮件地址）
    fn notice_receiver(&self) -> &str;

    fn new(config: Self::Config) -> Result<Self>;

    async fn sign(&self) -> Result<Self::Outcome>;

    fn success_msg(&self, _outcome: &Self::Outcome) -> String {
        let msg = format!("{} 签到成功", self.name());
        info!(
            "{} 签到成功 (user {}，{}",
            self.name(),
            self.notice_receiver(),
            msg
        );
        msg
    }

    fn success_body(&self, _outcome: &Self::Outcome) -> String {
        format!("{} 签到成功啦", self.name())
    }

    fn fail_msg(&self, e: &Error) -> String {
        let msg = format!("【签到失败】{} 签到失败，请手动补签！", self.name());
        warn!(
            "{} 签到失败 (user {})：{}",
            self.name(),
            self.notice_receiver(),
            msg
        );
        debug!("错误原因：{}", e);
        msg
    }

    fn fail_body(&self, e: &Error) -> String {
        format!("失败原因：{:?}", e)
    }
}

pub mod genshin;
pub mod nexus_pt;
pub mod sspanel;
pub mod v2ex;
