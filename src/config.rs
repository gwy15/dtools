//! Settings

use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{renamer, sign};
use anyhow::Context;

#[derive(Debug, Deserialize, Clone, Default)]
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
    pub sign: sign::Config,
    pub renamer: renamer::Config,
}

impl Config {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let f = File::open(path).context(format!("Settings file ({}) not found", path))?;
        let reader = BufReader::new(f);
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
