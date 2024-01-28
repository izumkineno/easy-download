use std::env::current_dir;
use byte_unit::{Byte, UnitType};
use easy_download::core::task::FileTask;
use anyhow::Result;
use tokio::time::Instant;
use easy_download::core::state::TaskState;
use easy_download::util::get_task;
// static URL: &str = "https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe";
// static URL: &str = "https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z";
// static URL: &str = "https://download-cdn.jetbrains.com.cn/rustrover/RustRover-233.13135.116.exe";
static URL: &str = "http://updates-http.cdn-apple.com/2019WinterFCS/fullrestores/041-39257/32129B6C-292C-11E9-9E72-4511412B0A59/iPhone_4.7_12.1.4_16D57_Restore.ipsw";

async fn download(resume: Option<Vec<u64>>) -> Result<()> {

    let thread= 8;
    let mut ft = FileTask::new(URL)?;
    // ft.set_proxy("http://127.0.0.1:10809");
    ft.set_thread(thread);
    let progress = ft.start(resume).await;


    let db = TaskState::new(ft.clone())?;
    db.save().unwrap();

    let mut progress_value = vec![0; thread as usize];
    let mut sum = 0;
    let size = ft.get_size();

    let mut t = 0u32;
    let mut last_size = 0;
    let mut last_time = Instant::now();

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
        if t == 10_1000 {
            t = 0;
            let _time = Instant::now().duration_since(last_time).as_micros();
            let _size = sum - last_size;
            let speed = _size as f64 / (_time as f64 / 100_1000.0);

            println!("{:.2}/s {:.2} {sum}-{} {:.2}% {:?}", Byte::from_u64(speed as u64).get_appropriate_unit(UnitType::Binary), Byte::from_u64(sum).get_appropriate_unit(UnitType::Binary), size, p * 100.0, progress_value);
            last_size = sum;
            last_time = Instant::now();
        }
        if sum >= size { break }
    }

    ft.merge().await?;
    ft.clear_temp().await?;

    Ok(())
}

async fn resume_task() -> Result<()> {

    let tmp = current_dir().unwrap().join("temp");

    let tss = get_task(tmp)?;
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

    // todo 处理线程数据因为网络波动卡死，动态分割，错误处理

    match resume_task().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }

    Ok(())
}
