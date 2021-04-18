#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

#[macro_use]
pub mod utils;

mod config;
mod notifier;
mod signers;

use anyhow::Result;
use config::Config;
use notifier::Notifier;
use signers::Signer;

async fn run<SignerImpl, Config, Outcome, It>(configs: It, notifier: &Notifier)
where
    It: IntoIterator<Item = Config>,
    SignerImpl: Signer<Config = Config, Outcome = Outcome>,
{
    for config in configs {
        let signer = SignerImpl::new(config);
        let signer = match signer {
            Ok(signer) => signer,
            Err(e) => {
                warn!("无法初始化 signer: {}", e);
                continue;
            }
        };
        let sign_result = signer.sign().await;

        let notice_result = match sign_result {
            Ok(outcome) => {
                notifier
                    .notify(
                        signer.notice_receiver(),
                        signer.success_msg(&outcome),
                        signer.success_body(&outcome),
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

    run::<signers::genshin::Signer, _, _, _>(config.genshin, &notifier).await;

    run::<signers::nexus_pt::Signer, _, _, _>(config.pt, &notifier).await;

    run::<signers::v2ex::Signer, _, _, _>(config.v2ex, &notifier).await;

    run::<signers::sspanel::Signer, _, _, _>(config.sspanel, &notifier).await;

    Ok(())
}
