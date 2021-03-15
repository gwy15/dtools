#[macro_use]
extern crate log;

mod config;
mod notifier;
mod sign;

use anyhow::Result;
use config::{Config, User};
use notifier::Notifier;
use sign::Signer;

async fn try_sign(cookies: String) -> Result<()> {
    Signer::new(cookies)?.sign().await
}

async fn handle_user(user: User, notifier: &Notifier) {
    match try_sign(user.cookies).await {
        Ok(_) => {
            notifier
                .notify(
                    user.email,
                    "原神签到成功".to_string(),
                    "原神签到成功啦".to_string(),
                )
                .await
        }
        Err(e) => {
            error!("签到失败: {}", e);
            notifier
                .notify(
                    user.email,
                    "【签到失败】原神签到失败，请手动补签".to_string(),
                    format!("失败原因：{:?}", e),
                )
                .await
        }
    }
    .unwrap_or_else(|e| {
        error!("发送邮件失败？？{:?}", e);
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file("./log4rs.yml", Default::default())?;
    debug!("logger initialized.");
    let config = Config::new()?;
    let notifier = Notifier::new(config.notification);
    for user in config.users {
        handle_user(user, &notifier).await;
    }
    Ok(())
}
