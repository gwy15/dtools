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
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, EnumString, AsRefStr, IntoStaticStr, EnumIter, Hash, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum TaskType {
    Genshin,
    NexusPt,
    SsPanel,
    V2ex,
}
impl TaskType {
    pub async fn run(self, config: &Config, notifier: &Notifier) {
        match self {
            TaskType::Genshin => {
                run::<signers::genshin::Signer, _, _, _>(config.genshin.iter().cloned(), notifier)
                    .await
            }
            TaskType::NexusPt => {
                run::<signers::nexus_pt::Signer, _, _, _>(config.pt.iter().cloned(), notifier).await
            }
            TaskType::SsPanel => {
                run::<signers::sspanel::Signer, _, _, _>(config.sspanel.iter().cloned(), notifier)
                    .await
            }
            TaskType::V2ex => {
                run::<signers::v2ex::Signer, _, _, _>(config.v2ex.iter().cloned(), notifier).await
            }
        }
    }
}

async fn run<SignerImpl, Config, Outcome, It>(configs: It, notifier: &Notifier)
where
    It: IntoIterator<Item = Config>,
    SignerImpl: Signer<Config = Config, Outcome = Outcome>,
    Config: Clone,
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
    let notifier = Notifier::new(config.notification.clone());

    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            clap::Arg::with_name("task")
                .short("t")
                .long("task")
                .possible_value("all")
                .possible_values(&TaskType::iter().map(|s| s.into()).collect::<Vec<_>>())
                .takes_value(true)
                .multiple(true)
                .help("Run the given task type"),
        )
        .get_matches();
    trace!("{:?}", matches);

    let mut tasks = std::collections::HashSet::new();
    for task in matches.values_of("task").unwrap_or_default() {
        debug!("task = {:?}", task);
        if task == "all" {
            for task in TaskType::iter() {
                tasks.insert(task);
            }
            break;
        }
        let task: TaskType = task.parse().unwrap();
        tasks.insert(task);
    }
    info!("Running tasks: {:?}", tasks);

    for task in tasks {
        task.run(&config, &notifier).await;
    }

    Ok(())
}
