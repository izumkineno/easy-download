
#[cfg(test)]
mod db {
    use easy_download::core::state::TaskState;
    use easy_download::core::task::FileTask;

    #[test]
    pub fn test_new() {
        let task = FileTask::new("https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe").unwrap();
        let mut db = TaskState::new(task).unwrap();
        dbg!(&db);
        db.save().unwrap();
        db.load().unwrap();
    }

}
