#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

mod config;
mod notifier;
pub mod sign;

use config::Config;
use notifier::Notifier;

use anyhow::{Context, Result};
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file("./log4rs.yml", Default::default()).context("./log4rs.yml not found")?;
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
                .possible_values(&sign::TaskType::iter().map(|s| s.into()).collect::<Vec<_>>())
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
            for task in sign::TaskType::iter() {
                tasks.insert(task);
            }
            break;
        }
        let task: sign::TaskType = task.parse().unwrap();
        tasks.insert(task);
    }
    info!("Running tasks: {:?}", tasks);
    for task in tasks {
        task.run(&config.sign, &notifier).await;
    }

    Ok(())
}
