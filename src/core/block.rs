use std::fmt::Display;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// 下载中(进度百分比)
    Downloading(u8),
    /// 下载完成
    Downloaded,
    /// 下载失败
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// 下载链接
    pub url: String,
    /// 块 id
    pub id: String,
    /// 开始位置
    pub start: u64,
    /// 结束位置
    pub end: u64,
    /// 大小
    pub size: u64,
    /// 状态
    pub status: DownloadStatus,
    /// 重试次数
    pub retry: u8,
    /// 最大重试次数
    pub max_retry: u8,
}

impl Block {
    pub fn new(url: impl AsRef<str>, id: String, start: u64, end: u64, max_retry: u8) -> Self {
        Self {
            url: url.as_ref().to_string(),
            id,
            start,
            end,
            size: end - start + 1,
            status: DownloadStatus::Downloading(0),
            retry: 0,
            max_retry,
        }
    }

}
