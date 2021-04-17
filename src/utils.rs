static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36";

pub fn client_builder() -> request::ClientBuilder {
    request::Client::builder().user_agent(USER_AGENT)
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
