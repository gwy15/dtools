//! 签到模块
#[macro_use]
pub mod utils;

mod config;
pub mod signers;

pub use config::Config;
use strum::{AsRefStr, EnumIter, EnumString, IntoStaticStr};

/// CLI 交互
#[derive(Debug, EnumString, AsRefStr, IntoStaticStr, EnumIter, Hash, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum TaskType {
    Genshin,
    NexusPt,
    SsPanel,
    V2ex,
}
impl TaskType {
    pub async fn run(self, config: &config::Config, notifier: &crate::Notifier) {
        match self {
            TaskType::Genshin => {
                run::<signers::genshin::Signer, _>(config.genshin.iter().cloned(), notifier).await
            }
            TaskType::NexusPt => {
                run::<signers::nexus_pt::Signer, _>(config.pt.iter().cloned(), notifier).await
            }
            TaskType::SsPanel => {
                run::<signers::sspanel::Signer, _>(config.sspanel.iter().cloned(), notifier).await
            }
            TaskType::V2ex => {
                run::<signers::v2ex::Signer, _>(config.v2ex.iter().cloned(), notifier).await
            }
        }
    }
}

async fn run<SignerImpl, It>(configs: It, notifier: &crate::Notifier)
where
    It: IntoIterator<Item = SignerImpl::Config>,
    SignerImpl: signers::Signer,
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
