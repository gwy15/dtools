#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde;

mod config;
mod notifier;
pub mod renamer;
pub mod sign;

use config::Config;
use notifier::Notifier;

use anyhow::Result;
use clap::Clap;
use std::collections::HashSet;
use strum::IntoEnumIterator;

#[derive(Debug, Clap)]
struct Opts {
    #[clap(short, long, default_value = "settings.toml")]
    config: String,

    #[clap(long, about = "Do not send email")]
    no_send: bool,

    #[clap(subcommand)]
    subcmd: SubCommand,
}
#[derive(Debug, Clap)]
enum SubCommand {
    #[clap(about = "Do sign-in")]
    Sign {
        #[clap(long, about = "Sign all tasks")]
        all: bool,

        #[clap(short, long)]
        task: Vec<sign::TaskType>,
    },
    #[clap(about = "Rename airport subscriptions")]
    Rename,
}

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file("./log4rs.yml", Default::default()).or_else(|_e| {
        pretty_env_logger::try_init()
            .map(|_| {
                debug!("log4rs not found, fallback to pretty_env_logger");
            })
            .map_err(|e| {
                eprintln!("Error init pretty_env_logger in fallback!");
                e
            })
    })?;
    debug!("logger initialized.");

    let opts: Opts = Opts::parse();

    let config = Config::new(&opts.config)?;
    let notifier = if opts.no_send {
        Notifier::noop()
    } else {
        Notifier::new(config.notification.clone())
    };

    match opts.subcmd {
        SubCommand::Sign { all, task: tasks } => {
            let tasks = if all {
                let mut tasks = HashSet::new();
                for task in sign::TaskType::iter() {
                    tasks.insert(task);
                }
                tasks
            } else {
                tasks.into_iter().collect()
            };
            for task in tasks {
                task.run(&config.sign, &notifier).await;
            }
        }
        SubCommand::Rename => {
            renamer::main(notifier, config.renamer).await?;
        }
    }

    Ok(())
}
