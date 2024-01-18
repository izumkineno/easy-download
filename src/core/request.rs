use std::fmt::Display;
use std::time::Duration;
use reqwest::header::{CONTENT_LENGTH, HeaderMap, USER_AGENT};
use anyhow::Result;
use reqwest::{Client, Proxy, Response};

pub struct FileRequest {
    url: String,
    headers: HeaderMap,
    proxy: Option<String>,
}

impl FileRequest {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self {
            url: url.as_ref().to_string(),
            headers: HeaderMap::new(),
            proxy: None,
        }
    }

    pub fn insert_header(mut self, key: impl AsRef<str> + reqwest::header::IntoHeaderName, value: impl AsRef<str>) -> Result<Self> {
        self.headers.insert(key, value.as_ref().parse()?);
        Ok(self)
    }

    pub fn create_client(&mut self) -> Result<Client> {
        // 没有浏览器标识就添加 Google浏览器 请求标识
        if !self.headers.contains_key(USER_AGENT) {
            self.headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());
        }
        // 解析代理
        let client = match &self.proxy {
            None => {
                Client::builder()
            }
            Some(p) => {
                Client::builder().proxy(Proxy::all(p)?)
            }
        };
        // 构建客户端
        let res = client.default_headers(self.headers.to_owned())
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(res)
    }

    pub async fn get(&mut self) -> Result<Response> {
        Ok(self.create_client()?
            .get(&self.url)
            .send()
            .await?
        )
    }

    pub async fn get_size(&mut self) -> Result<u64> {
        let res = self.get().await?;

        let file_size: u64 = match res.headers().get(CONTENT_LENGTH) {
            None => 0,
            Some(length) => length.to_str().unwrap().parse().unwrap(),
        };
        Ok(file_size)
    }
}
