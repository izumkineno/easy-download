use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use anyhow::{Result, format_err};
use byte_unit::{Byte, UnitType};
use reqwest::header::RANGE;
use tokio::fs::OpenOptions;
use crate::core::file_request::FileRequest;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::join;
use tokio::sync::mpsc::error::TryRecvError;


#[derive(Debug)]
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
    request: FileRequest,

    rx: Arc<Mutex<Option<Receiver<u64>>>>,
}

impl Block {
    pub fn new(start: u64, end: u64, dir: PathBuf, name: impl AsRef<str>, retry_max: u32, request: FileRequest) -> Self {
        Self {
            start,
            end,
            current: 0,
            dir,
            name: name.as_ref().to_string(),
            retry: 0,
            retry_max,
            request,
            rx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current + self.start >= self.end
    }

    pub fn get_progress(&self) -> Arc<Mutex<Option<Receiver<u64>>>> {
        self.rx.clone()
    }

    pub async fn request(&mut self) -> Result<()> {
        if self.request.headers.contains_key(RANGE) {
            self.request.headers.remove(RANGE);
        }
        self.request.insert_header(RANGE, format!("bytes={}-{}", &self.start, &self.end))?;
        match self.request.get().await {
            Ok(mut res) => {
                // 创建分块文件
                let file_path = self.dir.join(&self.name);
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .write(true)
                    .open(file_path)
                    .await?;

                let (tx, rx) = channel(48);
                self.rx.lock().unwrap().replace(rx);


                // 下载进度相关， 重要
                while let Some(chunk) = res.chunk().await? {
                    self.current += chunk.len() as u64;
                    // 成功写入或失败才会结束
                    let r = file.write_all(&chunk);
                    let t = tx.send(self.current);
                    join!(r, t);
                }
                // file.flush().await?;
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
