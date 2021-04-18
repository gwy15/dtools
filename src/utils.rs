static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36";

pub fn client_builder() -> request::ClientBuilder {
    request::Client::builder()
        .user_agent(USER_AGENT)
        .cookie_store(true)
}

macro_rules! header {
    ($($key:expr => $value:expr,)*) => {
        {
            let mut header = HeaderMap::new();
            $(
                header.append($key, $value.parse()?);
            )*
            header
        }
    };
}

pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}
