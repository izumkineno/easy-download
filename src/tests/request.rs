

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::Duration;
    use reqwest::header::RANGE;
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    use anyhow::Result;
    use byte_unit::{Byte, UnitType};
    use futures::StreamExt;
    use crate::core::FileRequest;

    static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";

    #[tokio::test]
    async fn get_size() {
        let mut request = FileRequest::new(URL);

        let adjusted_byte = request.get_size().await.unwrap();
        let adjusted_byte = Byte::from_u64(adjusted_byte).get_appropriate_unit(UnitType::Binary);

        print!("{adjusted_byte:.2}");
    }

    #[tokio::test]
    async fn get_block() -> Result<()> {
        let mut request = FileRequest::new(URL);
        let start: u64 = 0;
        let end: u64 = request.get_size().await?;
        let dir = std::env::current_dir()?;
        let name = "test";
        let tail = "exe";
        let mut size_current = 0;

        match request.insert_header(RANGE, format!("bytes={}-{}", start, end))?.get().await {
            Ok(res) => {
                let mut stream = res.bytes_stream();
                // 创建分块文件
                let file_path = dir.join(format!("{}.{}", name, tail));
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
                    size_current += block_size;

                    // 成功写入或失败才会结束
                    file.write_all_buf(&mut chunk).await?;

                    println!("download size: {:.2}, block size: {:.2}",
                        Byte::from_u64(size_current).get_appropriate_unit(UnitType::Binary),
                        Byte::from_u64(block_size).get_appropriate_unit(UnitType::Binary));
                }

            }
            Err(err) => {

            }
        }
        Ok(())
    }
}
