use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::fs::{File, remove_dir_all};
use tokio::io::AsyncWriteExt;
use tokio::spawn;
use tokio::sync::mpsc::Receiver;

use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};

use crate::core::block::Block;
use crate::core::file_request::FileRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTask {
    /// 文件名
    name: Option<String>,
    /// 尾缀
    tail: String,
    /// 下载链接
    url: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 代理
    proxy: Option<String>,
    /// 线程数
    thread: u8,
    /// 重试次数
    retry: u8,
    /// 缓存位置
    dir: PathBuf,
    /// 输出位置
    output: PathBuf,
    /// 文件大小
    size: u64,
}

impl FileTask {
    pub fn new(url: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            name: None,
            tail: "ed".to_string(),
            url: url.as_ref().to_string(),
            headers: HashMap::new(),
            dir: std::env::current_dir()?.join("temp"),
            output: std::env::current_dir()?.join("Downloads"),
            proxy: None,
            thread: 2,
            retry: 5,
            size: 0,
        })
    }

    pub fn set_proxy(&mut self, proxy: impl AsRef<str>) {
        self.proxy = Some(proxy.as_ref().to_string())
    }

    pub fn set_thread(&mut self, thread: u8) {
        self.thread = thread
    }

    pub fn get_thread(&self) -> u8 {
        self.thread
    }

    pub fn set_tail(&mut self, tail: impl AsRef<str>) {
        self.tail = tail.as_ref().to_string()
    }

    pub fn insert_header(&mut self, key: impl AsRef<str> + reqwest::header::IntoHeaderName, value: impl AsRef<str>) -> anyhow::Result<()> {
        self.headers.insert(String::from(key.as_ref()), value.as_ref().parse()?);
        Ok(())
    }

    pub fn set_retry(&mut self, retry: u8) {
        self.retry = retry
    }

    pub fn set_tmp_dir(&mut self, dir: impl AsRef<Path>) {
        self.dir = dir.as_ref().to_owned()
    }

    pub fn set_output(&mut self, dir: impl AsRef<Path>) {
        self.output = dir.as_ref().to_owned()
    }

    pub fn filename(&self) -> &str {
        self.url
            .split('/')
            .last()
            .and_then(|i| i.split('?').next())
            .unwrap()
    }

    pub fn get_name(&self) -> &str {
        let mut name = self.filename();
        if let Some(n) = &self.name {
            name = n;
        }
        name
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_tmp_dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn get_output(&self) -> &PathBuf {
        &self.output
    }
}

impl FileTask {
    pub async fn start(&mut self, resume: Option<Vec<u64>>) -> Result<Vec<Arc<Mutex<Option<Receiver<u64>>>>>> {
        let resume_flag = resume.is_some();
        let resume = resume.unwrap_or_default();

        self.dir = self.dir.join(self.get_name());
        // 创建文件路径
        create_dir_all(&self.dir)?;
        create_dir_all(&self.output)?;
        // 创建基础请求
        let mut req = FileRequest::new(&self.url);
        req.headers = self.headers.clone().iter().map(|(key, value)| (key.parse().unwrap(), value.parse().unwrap())).collect();
        if let Some(p) = self.proxy.as_ref() {
            req.set_proxy(p)
        }

        // 获取文件大小
        let (size, is_resumed) = req.get_size().await?;
        self.size = size;
        if !is_resumed {
            self.thread = 1;
        }

        let block_size = self.size / self.thread as u64;
        let mut blocks_handle = Vec::with_capacity(self.thread as usize);
        let mut progress = Vec::with_capacity(self.thread as usize);

        for i in 0..self.thread {
            let start = block_size * (i as u64);
            let end = if i == self.thread - 1 { self.size - 1 } else { block_size * ((i + 1) as u64) - 1 };

            let mut block = Block::new(
                start,
                end,
                &self.dir,
                format!("{}.{}{i}", &self.get_name(), &self.tail),
                self.retry,
                req.clone()).await?;

            if resume_flag {
                let v = resume[i as usize];
                block.set_current(v);
            }

            // 进度
            let r = block.get_progress();
            progress.push(r);

            let handle = spawn(async move {
                let _ = block.request().await;
            });

            blocks_handle.push(handle);
        }

        Ok(progress)
    }

    pub fn get_blocks_file(&self) -> Result<Vec<PathBuf>> {
        let mut file_tmp_path = Vec::with_capacity(self.thread as usize);
        for i in 0..self.thread {
            let path = self.dir.clone().join(format!("{}.{}{}", self.get_name(), &self.tail, i));
            if path.is_file() {
                file_tmp_path.push(path);
            } else {
                return Err(format_err!("未获取到块文件: {}", path.display()))
            }
        }
        Ok(file_tmp_path)
    }

    pub async fn merge(&mut self) -> Result<()> {
        let save_path = self.output.join(self.get_name());

        let file_tmp_path = self.get_blocks_file()?;

        dbg!(&file_tmp_path);
        dbg!(&save_path);

        let mut f = vec![];
        for path in file_tmp_path {
            if std::fs::metadata(&path)?.len() >= self.size / self.thread as u64 {
                f.extend(std::fs::read(path)?);
            } else {
                return Err(format_err!("文件不完整: {}", path.display()))
            }

        }
        let mut file = File::create(save_path).await?;
        file.write_all(&f).await?;
        Ok(())
    }

    pub async fn clear_temp(&mut self) -> Result<()> {
        Ok(remove_dir_all(&self.dir).await?)
    }

}
