use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::join;

use anyhow::{Result, format_err};
use reqwest::header::RANGE;

use crate::core::file_request::FileRequest;


#[derive(Debug)]
pub struct Block {
    /// 块文件句柄
    file: File,
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
    retry: u8,
    /// 重试最大次数
    retry_max: u8,
    /// 请求句柄
    request: FileRequest,
    /// 进度传输通道
    rx: Arc<Mutex<Option<Receiver<u64>>>>,
    /// 进度缓冲区大小
    rx_size: usize,
}

impl Block {
    pub async fn new(start: u64, end: u64, dir: impl AsRef<Path>, name: impl AsRef<str>, retry_max: u8, request: FileRequest) -> Result<Self> {
        let dir = dir.as_ref().to_owned();
        let name = name.as_ref().to_owned();
        // 创建分块文件
        let file_path = dir.join(&name);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(file_path)
            .await?;
        Ok(
            Self {
                file,
                start,
                end,
                current: 0,
                dir,
                name,
                retry: 0,
                retry_max,
                request,
                rx: Arc::new(Mutex::new(None)),
                rx_size: 20480,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.current + self.start >= self.end
    }

    pub fn get_progress(&self) -> Arc<Mutex<Option<Receiver<u64>>>> {
        self.rx.clone()
    }

    pub fn set_rx_size(&mut self, rx_size: usize) {
        self.rx_size = rx_size
    }

    pub fn set_current(&mut self, current: u64) {
        self.current = current
    }

}

impl Block {
    async fn download(&mut self, mut res: reqwest::Response) -> Result<()> {
        self.retry = 0;

        // 缓存过低会导致传输中断
        let (tx, rx) = channel(self.rx_size);
        self.rx.lock().unwrap().replace(rx);

        println!("start download: {}", &self.name);


        while let Some(chunk) = res.chunk().await? {
            let mut chunk = chunk;
            self.current += chunk.len() as u64;
            // 成功写入或失败才会结束
            let r = self.file.write_all_buf(&mut chunk);
            let t = tx.send_timeout(self.current, Duration::from_nanos(10));
            join!(r, t);
        }
        println!("downloaded: {} , {} - {}", self.name, self.end - self.start, self.current );
        self.file.flush().await?;
        Ok(())
    }

    async fn download_error_handler(&mut self, err: anyhow::Error) -> Result<()> {
        println!("download retry {}: {}", self.retry, err);
        self.retry += 1;
        if self.retry > self.retry_max {
            println!("download retry max out: {}", err);
            return Err(format_err!("download retry max out: {}", err));
        }
        tokio::time::sleep(Duration::from_secs((3 * self.retry) as u64)).await;
        Box::pin(self.request()).await?;
        Ok(())
    }

    pub async fn request(&mut self) -> Result<()> {
        self.request.insert_header(RANGE, format!("bytes={}-{}", self.start + self.current, self.end))?;
        dbg!(&self.request);
        match self.request.get().await {
            Ok(res) => {
                self.download(res).await?;
            }
            Err(err) => {
                self.download_error_handler(err).await?;
            }
        }
        Ok(())
    }
}
