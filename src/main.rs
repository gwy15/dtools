#[macro_use]
extern crate log;

mod config;
mod notifier;
mod signers;

use anyhow::Result;
use config::Config;
use notifier::Notifier;
use signers::Signer;

async fn run<S, Config, It>(configs: It, notifier: &Notifier)
where
    It: Iterator<Item = Config>,
    S: Signer<Config = Config>,
{
    for config in configs {
        let signer = S::new(config);
        let signer = match signer {
            Ok(signer) => signer,
            Err(e) => {
                warn!("无法初始化 signer: {}", e);
                continue;
            }
        };
        let sign_result = signer.sign().await;

        let notice_result = match sign_result {
            Ok(_) => {
                notifier
                    .notify(
                        signer.notice_receiver(),
                        signer.success_msg(),
                        signer.success_body(),
                    )
                    .await
            }
            Err(e) => {
                notifier
                    .notify(
                        signer.notice_receiver(),
                        signer.fail_msg(&e),
                        signer.fail_body(&e),
                    )
                    .await
            }
        };
        if let Err(err) = notice_result {
            error!("发送邮件失败：{:?}", err);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file("./log4rs.yml", Default::default())?;
    debug!("logger initialized.");
    let config = Config::new()?;
    let notifier = Notifier::new(config.notification);

    run::<signers::genshin::Signer, signers::genshin::Config, _>(
        config.genshin.into_iter(),
        &notifier,
    )
    .await;

    Ok(())
}
