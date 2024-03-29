use std::time::Duration;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, HeaderMap, USER_AGENT};
use anyhow::Result;
use reqwest::{Client, Proxy, Response};

#[derive(Debug, Clone)]
pub struct FileRequest {
    url: String,
    pub headers: HeaderMap,
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

    pub fn insert_header(&mut self, key: impl AsRef<str> + reqwest::header::IntoHeaderName, value: impl AsRef<str>) -> Result<()> {
        self.headers.insert(key, value.as_ref().parse()?);
        Ok(())
    }

    pub fn set_proxy(&mut self, proxy: impl AsRef<str>) {
        self.proxy = Some(proxy.as_ref().to_string())
    }

}

impl FileRequest {

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

    pub async fn get_size(&mut self) -> Result<(u64, bool)> {
        let res = self.get().await?;
        let headers = res.headers();

        let file_size: u64 = match headers.get(CONTENT_LENGTH) {
            None => 0,
            Some(length) => length.to_str().unwrap().parse().unwrap(),
        };
        let is_resumed = headers.contains_key(ACCEPT_RANGES) || headers.contains_key(CONTENT_RANGE);
        Ok((file_size, is_resumed))
    }
}
