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
use futures::future::join_all;
use tokio::sync::mpsc::error::TryRecvError;
use easy_download::core::task::FileTask;
use anyhow::Result;

// static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";
// static URL: &str = "https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z";
static URL: &str = "https://download.jetbrains.com/rustrover/RustRover-233.13135.116.exe";
// static URL: &str = "http://updates-http.cdn-apple.com/2019WinterFCS/fullrestores/041-39257/32129B6C-292C-11E9-9E72-4511412B0A59/iPhone_4.7_12.1.4_16D57_Restore.ipsw";

#[tokio::main]
async fn main() -> Result<()> {

    let thread= 12;
    let mut ft = FileTask::new(URL)?;
    // ft.set_proxy("http://127.0.0.1:10809");
    ft.set_thread(thread);
    let progress = ft.start().await;

    let mut progress_value = vec![0; thread as usize];
    let mut sum = 0;
    loop {
        if let Ok(progress) = progress.as_ref() {
            for (i, p) in progress.iter().enumerate() {
                let mut r = p.lock().unwrap();
                if let Some(res) = r.as_mut() {
                    while let Ok(v) = res.try_recv() {
                        progress_value[i] = v;
                    }
                }
            }
        };
        sum = progress_value.iter().sum::<u64>();
        let p = sum as f64 / ft.get_size() as f64;
        println!("{:.2}  {sum}-{} {:.2}% {:?}", Byte::from_u64(sum).get_appropriate_unit(UnitType::Binary), ft.get_size(), p * 100.0, progress_value);
        if sum >= ft.get_size() { break }
    }

    ft.merge().await?;
    ft.remove().await?;


    // // todo 然后处理持久化断线重连，再处理线程数据卡死
    //
    //
    // let mut file_tmp_path = Vec::with_capacity(thread);
    // for i in 0..thread {
    //     let path = dir.clone().join(format!("{}.{}{}", name, tail, i));
    //     if path.is_file() {
    //         file_tmp_path.push(path);
    //     } else {
    //         eprintln!("file not exist: {}", path.display());
    //     }
    // }
    //
    // let mut f = vec![];
    // for path in file_tmp_path {
    //     f.extend(std::fs::read(path)?);
    // }
    // let mut file = File::create(output.join(name)).await?;
    // file.write_all(&f).await?;


    // remove_dir_all(dir)?;

    Ok(())
}
