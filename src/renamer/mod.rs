mod config;
mod parse;

pub use config::Config;

use anyhow::Result;
use config::Subscription;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::{
    fs,
    io::{AsyncWriteExt, BufWriter},
};

use crate::notifier::Notifier;

async fn run(
    output: PathBuf,
    subscriptions: Vec<Subscription>,
) -> Result<BTreeMap<String, Result<usize>>> {
    let mut results = BTreeMap::new();
    let mut nodes = vec![];

    for sub in subscriptions {
        macro_rules! check {
            ($r:expr) => {
                match $r {
                    Ok(ok) => ok,
                    Err(e) => {
                        warn!("机场 {} 失败：{:?}", sub.name, e);
                        results.insert(sub.name.clone(), Err(e));
                        continue;
                    }
                }
            };
        }
        let content = check!(sub.get().await);
        let mut airport = check!(parse::Airport::new(&sub.name, content));
        check!(airport.rename(&sub.replacements));

        // output
        info!("{} has {} nodes.", airport.name, airport.nodes.len());
        results.insert(airport.name, Ok(airport.nodes.len()));
        nodes.extend(airport.nodes);
    }
    // write
    let nodes_encoded = base64::encode(
        nodes
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let mut output_file = BufWriter::new(fs::File::create(output).await?);
    output_file.write_all(nodes_encoded.as_bytes()).await?;

    info!("done.");
    Ok(results)
}

pub async fn main(notifier: Notifier, config: Config) -> Result<()> {
    let results = run(config.output, config.subscriptions).await;

    match results {
        Err(e) => {
            notifier
                .notify(
                    &config.receiver,
                    "转换订阅失败！",
                    format!("转换订阅失败：{:?}", e),
                )
                .await?;
        }
        Ok(results) => {
            let total_nodes: usize = results.values().filter_map(|r| r.as_ref().ok()).sum();
            let total_failed = results.values().filter(|r| r.is_err()).count();

            let title = if total_failed == 0 {
                format!("转换订阅链接成功，共 {} 个订阅", total_nodes)
            } else {
                format!(
                    "转换订阅链接部分成功，共 {} 个订阅，{} 个机场失败",
                    total_nodes, total_failed
                )
            };

            let ok = results
                .iter()
                .filter_map(|(k, v)| Some((k, v.as_ref().ok()?)));
            let err = results
                .iter()
                .filter_map(|(k, v)| Some((k, v.as_ref().err()?)));

            let mut body = String::new();
            for (name, cnt) in ok {
                body += &format!("机场 {} 成功，共 {} 个节点\n", name, cnt);
            }
            body += "\n\n";
            for (name, e) in err {
                body += &format!("机场 {} 失败：{}\n详细原因：{:?}\n\n", name, e, e);
            }

            if std::env::var("DRYRUN").is_err() {
                notifier.notify(&config.receiver, title, body).await?;
            }
        }
    }

    Ok(())
}
