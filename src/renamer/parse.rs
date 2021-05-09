use anyhow::{anyhow, bail, Context, Error, Result};
use std::{collections::HashMap, str::FromStr};

fn atob(s: &str, config: base64::Config) -> Result<String> {
    let decoded = base64::decode_config(s, config).context("invalid base64")?;
    let s = String::from_utf8(decoded).context("The decoded data is not valid utf8.")?;
    Ok(s)
}

#[derive(Debug)]
pub struct Airport {
    pub name: String,
    pub nodes: Vec<Node>,
    node_name_cnt: HashMap<String, u32>,
}
impl Airport {
    pub fn new(name: impl Into<String>, encoded: impl AsRef<str>) -> Result<Self> {
        let decoded = atob(encoded.as_ref(), base64::STANDARD)?;

        let mut nodes = vec![];

        for line in decoded.lines() {
            let node: Node = line.parse()?;
            nodes.push(node);
        }

        Ok(Self {
            name: name.into(),
            nodes,
            node_name_cnt: HashMap::new(),
        })
    }

    fn new_name(&mut self, mut name: String, replacements: &[regex::Regex]) -> Option<String> {
        // 筛选关键词
        const KWS: &[&str] = &[
            "剩余", "规则", "购买", "收入", "流量", "过期", "链接", "官网", "域名",
        ];
        if KWS.iter().any(|kw| name.contains(kw)) {
            return None;
        }
        //
        for r in replacements {
            name = r.replace_all(&name, "").to_string();
        }
        name = name.trim().to_string();

        // 解决重名
        let new_name = match self.node_name_cnt.get_mut(&name) {
            Some(times) => {
                *times += 1;
                format!("{}({}) - {}", name, times, self.name)
            }
            None => {
                self.node_name_cnt.insert(name.to_string(), 1);
                format!("{} - {}", name, self.name)
            }
        };
        Some(new_name)
    }

    pub fn rename(&mut self, replacements: &[String]) -> Result<()> {
        // 先编译正则
        let regexps = replacements
            .iter()
            .map(|r| regex::Regex::new(r))
            .collect::<Result<Vec<regex::Regex>, _>>()?;

        let nodes = std::mem::take(&mut self.nodes);

        for mut node in nodes {
            let name = node.name()?.to_string();
            trace!("raw name = {:?}", name);
            if let Some(new_name) = self.new_name(name, &regexps) {
                debug!("new_name = {:?}", new_name);
                node.set_name(new_name)?;
                self.nodes.push(node);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Node {
    // vmess 协议，是个 json
    VMess {
        inner: serde_json::Value,
    },
    // ssr 协议，path 和 query
    Ssr {
        path: String,
        query: HashMap<String, String>,
    },
}
impl Node {
    pub fn name(&self) -> Result<String> {
        match self {
            Node::VMess { inner } => {
                const NAME: &str = "ps";
                let name = inner
                    .get(NAME)
                    .ok_or_else(|| anyhow!("vmess name not found."))?
                    .as_str()
                    .ok_or_else(|| anyhow!("name is not string"))?
                    .trim()
                    .to_string();

                Ok(name)
            }
            Node::Ssr { query, .. } => {
                let remarks = query
                    .get("remarks")
                    .ok_or_else(|| anyhow!("ssr: remarks not found"))?;
                // decode
                trace!("remarks = {:?}", remarks);
                let remarks = atob(remarks, base64::URL_SAFE_NO_PAD)?.trim().to_string();

                Ok(remarks)
            }
        }
    }

    fn set_name(&mut self, name: String) -> Result<()> {
        match self {
            Node::VMess { inner } => {
                const NAME: &str = "ps";
                inner[NAME] = serde_json::Value::String(name);
            }
            Node::Ssr { query, .. } => {
                query.insert(
                    "remarks".to_string(),
                    base64::encode_config(name, base64::URL_SAFE_NO_PAD),
                );
            }
        }
        Ok(())
    }

    fn from_vmess(body: &str) -> Result<Self> {
        let parsed = atob(body, base64::STANDARD)?;
        let config: serde_json::Value = serde_json::from_str(&parsed)?;
        Ok(Self::VMess { inner: config })
    }

    fn from_ssr(body: &str) -> Result<Self> {
        // 有的链接会用 _ 代替 ?，不知道为啥
        let s = atob(body, base64::URL_SAFE_NO_PAD)?;
        let mut iter = s.split("?");
        let path = iter
            .next()
            .ok_or_else(|| anyhow!("? not found in url"))?
            .to_string();
        let query_s = iter
            .next()
            .ok_or_else(|| anyhow!("? not found in url"))?
            .to_string();

        trace!("ssr url: path={:?} query={:?}", path, query_s);

        // parse query
        let mut query = HashMap::new();
        for pair in query_s.split("&") {
            let mut iter = pair.split("=");
            let key = iter
                .next()
                .ok_or_else(|| anyhow!("= not found in segment"))?;
            let value = iter.next().unwrap_or_default();
            query.insert(key.to_string(), value.to_string());
        }

        Ok(Self::Ssr { path, query })
    }
}

impl FromStr for Node {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split("://").collect();
        if split.len() != 2 {
            bail!("include multiple ://")
        }
        let protocol = split[0];
        let body = split[1];
        // test protocol
        match protocol {
            "vmess" => Self::from_vmess(body),
            "ssr" => Self::from_ssr(body),
            _ => {
                println!("{:?} {:?}", protocol, body);
                bail!("Unsupported protocol: {}", protocol)
            }
        }
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self {
            Node::VMess { inner } => format!(
                "vmess://{}",
                base64::encode(serde_json::to_string(&inner).unwrap())
            ),
            Node::Ssr { path, query } => {
                let query: String = query
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                let url = format!("{}?{}", path, query);
                format!("ssr://{}", base64::encode_config(url, base64::URL_SAFE))
            }
        }
    }
}
