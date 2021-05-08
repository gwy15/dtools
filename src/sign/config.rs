use super::signers;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub genshin: Vec<signers::genshin::Config>,
    #[serde(default)]
    pub pt: Vec<signers::nexus_pt::Config>,
    #[serde(default)]
    pub v2ex: Vec<signers::v2ex::Config>,
    #[serde(default)]
    pub sspanel: Vec<signers::sspanel::Config>,
}
