

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use byte_unit::{Byte, UnitType};
    use tokio::fs::File;
    use tokio::io::{AsyncWriteExt};
    use crate::core::file_request::FileRequest;

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
        // todo 合并失败，不知道是分块失败还是合并错误
        // let mut request = FileRequest::new(URL);
        // let start: u64 = 0;
        // let end: u64 = request.get_size().await?;
        // let dir = std::env::current_dir()?;
        // let dir2 = std::env::current_dir()?;
        // let name = "test.t1";
        // let name2 = "test.t2";
        // let retry_max = 5;
        // let req = Arc::new(Mutex::new(request));
        //
        // let mut block = Block::new(start, end / 2 + 1, dir, name, retry_max, req.clone());
        // let mut block2 = Block::new(end / 2, end, dir2, name2, retry_max, req.clone());
        // join!(block.request(), block2.request());


        let f1 = std::fs::read("test.t1").unwrap();
        let f2 = std::fs::read("test.t1").unwrap();

        let mut f = vec![];
        f.extend(f1);
        f.extend(f2);

        let mut file = File::create("test.exe").await?;
        file.write_all(f.as_slice()).await?;

        Ok(())
    }
}
