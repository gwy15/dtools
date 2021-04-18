//! Settings

use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::signers;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Notification {
    pub sender: String,
    pub pswd: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub master: String,
    pub notification: Notification,
    #[serde(default)]
    pub genshin: Vec<signers::genshin::Config>,
    #[serde(default)]
    pub pt: Vec<signers::nexus_pt::Config>,
    #[serde(default)]
    pub v2ex: Vec<signers::v2ex::Config>,
    #[serde(default)]
    pub sspanel: Vec<signers::sspanel::Config>,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let reader = BufReader::new(File::open("./sign_settings.toml").unwrap());
        Ok(Self::from_reader(reader)?)
    }
}

impl Config {
    pub fn from_reader(mut f: impl Read) -> anyhow::Result<Self> {
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        Self::from_str(&buf)
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

#[test]
pub fn test_config() {
    let s = r#"
master = "admin@example.com"
[notification]
sender = "example@example.com"
pswd = "pswd"
host = "example.com"
port = 1234

[[genshin]]
cookies = "123"
email = "123@example.com"

    "#;
    let c = Config::from_str(s).unwrap();
    assert_eq!(c.notification.sender, "example@example.com");
    assert_eq!(c.genshin.len(), 1);
    assert_eq!(c.genshin[0].email, "123@example.com");
}
