use std::fs::remove_dir_all;
use std::path::Path;
use byte_unit::{Byte, UnitType};
use easy_download::core::task::FileTask;
use anyhow::Result;
use easy_download::core::state::TaskState;

// static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";
// static URL: &str = "https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z";
// static URL: &str = "https://download.jetbrains.com/rustrover/RustRover-233.13135.116.exe";
// static URL: &str = "http://updates-http.cdn-apple.com/2019WinterFCS/fullrestores/041-39257/32129B6C-292C-11E9-9E72-4511412B0A59/iPhone_4.7_12.1.4_16D57_Restore.ipsw";
static URL: &str = "https://issuepcdn.baidupcs.com/issue/netdisk/yunguanjia/BaiduNetdisk_7.37.5.3.exe";

async fn download(resume: Option<Vec<u64>>) -> Result<()> {

    let thread= 8;
    let mut ft = FileTask::new(URL)?;
    // ft.set_proxy("http://127.0.0.1:10809");
    ft.set_thread(thread);
    let progress = ft.start(resume).await;

    dbg!(&ft);

    let db = TaskState::new(ft.clone())?;
    db.save().unwrap();

    let mut progress_value = vec![0; thread as usize];
    let mut sum = 0;
    let size = ft.get_size();

    let mut t = 0u32;

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
        let p = sum as f64 / size as f64;
        t += 1;
        if t == 100_1000 {
            t = 0;
            println!("{:.2}  {sum}-{} {:.2}% {:?}", Byte::from_u64(sum).get_appropriate_unit(UnitType::Binary), size, p * 100.0, progress_value);
        }
        if sum >= size { break }
    }

    ft.merge().await?;
    ft.remove().await?;


    Ok(())
}

fn get_task_one(tmp: impl AsRef<Path>) -> Result<Vec<TaskState>> {
    let mut tss = vec![];
    let path = Path::new(tmp.as_ref());
    let dirs = path.read_dir()?.collect::<Result<Vec<_>, _>>()?;
    for dir in dirs {
        let d = dir.path().join("state");
        let db = sled::Config::default()
            .path(&d)
            .segment_size(1024)
            .open()?;
        let size = db.size_on_disk()?;
        if size == 0 {
            remove_dir_all(d)?;
        } else {
            let ts = TaskState::from(db)?;
            tss.push(ts);
        }
    }
    Ok(tss)
}

async fn resume_task() -> Result<()> {

    let tmp = "E:\\izum\\code\\easy-download\\temp";

    let tss = get_task_one(tmp)?;
    let mut resume_vec = vec![];
    for x in tss {
        let mut resume = vec![];
        let blocks = x.task.get_blocks_file()?;
        dbg!(&blocks);
        for b in blocks {
            let block_size = std::fs::metadata(&b)?.len();
            dbg!(block_size);
            resume.push(block_size);
        }
        resume_vec.push(resume);
    }

    if resume_vec.len() == 0 {
        download(None).await?;
    } else {
        download(Some(resume_vec[0].clone())).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    // todo 处理线程数据卡死， 频繁中断后导致合并后的文件异常
    resume_task().await?;

    Ok(())
}
