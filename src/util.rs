use std::fs::remove_dir_all;
use std::path::Path;
use crate::core::state::TaskState;

/// 读取临时文件夹内的下载状态数据
pub fn get_task(tmp: impl AsRef<Path>) -> anyhow::Result<Vec<TaskState>> {
    let mut tss = vec![];
    let path = Path::new(tmp.as_ref());
    let dirs = path.read_dir()?.collect::<anyhow::Result<Vec<_>, _>>()?;
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
