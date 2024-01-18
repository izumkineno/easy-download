use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, format_err};
use byte_unit::{Byte, UnitType};
use reqwest::header::RANGE;
use tokio::fs::OpenOptions;
use crate::core::file_request::FileRequest;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Block {
    /// 开始位置
    start: u64,
    /// 结束位置
    end: u64,
    /// 当前位置
    current: u64,
    /// 缓存文件位置
    dir: PathBuf,
    /// 块文件名, 带尾缀
    name: String,
    /// 当前重试次数
    retry: u32,
    /// 重试最大次数
    retry_max: u32,
    /// 请求句柄
    request: Arc<Mutex<FileRequest>>,
}

impl Block {
    pub fn new(start: u64, end: u64, dir: PathBuf, name: impl AsRef<str>, retry_max: u32, request: Arc<Mutex<FileRequest>>) -> Self {
        Self {
            start,
            end,
            current: 0,
            dir,
            name: name.as_ref().to_string(),
            retry: 0,
            retry_max,
            request,
        }
    }

    pub async fn request(&mut self) -> Result<()> {
        let mut req = self.request.lock().await;
        if req.headers.contains_key(RANGE) {
            req.headers.remove(RANGE);
        }
        req.insert_header(RANGE, format!("bytes={}-{}", &self.start, &self.end))?;
        match req.get().await {
            Ok(res) => {
                let mut stream = res.bytes_stream();
                // 创建分块文件
                let file_path = self.dir.join(&self.name);
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .write(true)
                    .open(file_path)
                    .await?;

                // 下载进度相关， 重要
                let mut block_size = 0;
                while let Some(chunk) = stream.next().await {
                    let mut chunk = chunk?;

                    block_size = chunk.len() as u64;
                    self.current += block_size;

                    // 成功写入或失败才会结束
                    file.write_all_buf(&mut chunk).await?;

                    println!("download size: {:.2}, block size: {:.2}",
                             Byte::from_u64(self.current).get_appropriate_unit(UnitType::Binary),
                             Byte::from_u64(block_size).get_appropriate_unit(UnitType::Binary));
                }
            }
            Err(err) => {
                self.retry += 1;
                if self.retry > self.retry_max {
                    return Err(format_err!("download retry max out: {}", err));
                }
            }
        }
        Ok(())
    }
}
