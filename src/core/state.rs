use std::fs::create_dir_all;
use sled::Db;
use anyhow::Result;

use crate::core::task::FileTask;

#[derive(Debug)]
pub struct TaskState {
    db: Db,
    pub task: FileTask,
    pub pause: bool,
}

impl TaskState {
    pub fn new(file_task: FileTask) -> Result<Self>{
        let path = file_task.get_tmp_dir().join("state");
        create_dir_all(&path)?;
        let db = sled::Config::default()
            .path(path)
            .segment_size(1024)
            .open()?;
        Ok(
            Self {
            db,
            task: file_task,
            pause: false,
        })
    }

    pub fn set_pause(&mut self, pause: bool) {
        self.pause = pause;
    }
}

impl TaskState {
    pub fn from(db: Db) -> Result<Self> {
        let mut ts = Self {
            db,
            task: FileTask::new("")?,
            pause: false,
        };
        ts.load()?;
        Ok(ts)
    }

    pub fn save(&self) -> Result<()> {
        self.db.insert("task", serde_json::to_string(&self.task)?.as_bytes())?;
        self.db.insert("pause", serde_json::to_string(&self.pause)?.as_bytes())?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        self.task = serde_json::from_slice(&self.db.get("task")?.unwrap())?;
        self.pause = serde_json::from_slice(&self.db.get("pause")?.unwrap())?;
        Ok(())
    }
}