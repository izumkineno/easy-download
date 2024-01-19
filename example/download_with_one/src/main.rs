use std::error::Error;
use std::fs::{create_dir_all, remove_dir_all};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::{join, spawn};
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use easy_download::core::block::Block;
use easy_download::core::file_request::FileRequest;
use byte_unit::{Byte, UnitType};
use tokio::sync::mpsc::error::TryRecvError;

// static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";
static URL: &str = "https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z";
// static URL: &str = "https://download.jetbrains.com/rustrover/RustRover-233.13135.116.exe";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut request = FileRequest::new(URL);
    request.set_proxy("http://127.0.0.1:10809");
    let start: u64 = 0;
    let end: u64 = request.get_size().await?;
    let dir = std::env::current_dir()?.join("temp");
    let name = "test.t1";
    let name2 = "test.t2";
    let retry_max = 5;
    let block_size = end / 2;

    // 创建文件路径
    create_dir_all(dir.clone())?;

    let mut block = Block::new(start, block_size - 1, dir.clone(), name, retry_max, request.clone());
    let mut block2 = Block::new(block_size, end, dir.clone(), name2, retry_max, request);


    // todo 合并
    let r = block.get_progress();
    thread::spawn(move || loop {
        {
            let mut r = r.lock().unwrap();
            if let Some(res) = r.as_mut() {
                match res.try_recv() {
                    Ok(v) => {
                        let byte = Byte::from_u64(v);
                        let adjusted_byte = byte.get_appropriate_unit(UnitType::Binary);
                        println!("{adjusted_byte:.2}");
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                }
            }
        }


    });

    let r = block2.get_progress();
    thread::spawn(move || loop {
        {
            let mut r = r.lock().unwrap();
            if let Some(res) = r.as_mut() {
                match res.try_recv() {
                    Ok(v) => {
                        let byte = Byte::from_u64(v);
                        let adjusted_byte = byte.get_appropriate_unit(UnitType::Binary);
                        println!("{adjusted_byte:.2}");
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                }
            }
        }

    });

    join!(block.request(), block2.request());


    let f1 = std::fs::read(dir.clone().join("test.t1")).unwrap();
    let f2 = std::fs::read(dir.clone().join("test.t2")).unwrap();

    let mut f = vec![];
    f.extend(f1);
    f.extend(f2);

    let mut file = File::create("test.exe").await?;
    file.write_all(&f).await?;

    remove_dir_all(dir)?;

    Ok(())
}
