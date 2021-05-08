//! Settings

use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::sign;
use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
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
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let reader = BufReader::new(
            File::open("./settings.toml").context("Settings file (settings.toml) not found")?,
        );
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
