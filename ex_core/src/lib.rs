use chrono::prelude::*;
use jwalk::WalkDir;
use std::{
    env, fs,
    io::{self},
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Ex {
    files: Vec<PathBuf>,
}

impl Ex {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.set_directory(Path::new("C:\\"));
        s
    }

    pub fn current_path(&self) -> &Path {
        &self.files[0]
    }

    pub fn current_path_string(&self) -> String {
        self.files[0].to_string_lossy().to_string()
    }

    pub fn current_file(&self) -> String {
        self.files[0]
            .file_name()
            .unwrap_or_else(|| self.files[0].as_os_str())
            .to_string_lossy()
            .to_string()
    }

    pub fn set_directory(&mut self, path: &Path) {
        if env::set_current_dir(path).is_ok() {
            let mut files: Vec<PathBuf> = WalkDir::new(&path)
                .max_depth(1)
                .skip_hidden(false)
                .into_iter()
                .flatten()
                .map(|dir| dir.path())
                .collect();

            files.sort_by_key(|a| !a.is_dir());

            self.files = files;
        };
    }

    pub fn previous_dir(&mut self) {
        if let Some(path) = self.files.first().cloned() {
            if let Some(parent) = path.parent() {
                self.set_directory(parent);
            }
        }
    }

    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    //TODO: too slow on windows
    pub fn file_size(path: &Path) -> Option<String> {
        if let Ok(metadata) = path.metadata() {
            let size = metadata.file_size();
            let size_str = if size < 1_000 {
                if size == 0 {
                    String::from("0 KB")
                } else {
                    String::from("1 KB")
                }
            } else if size < 1_000_000 {
                format!("{} KB", size / 1_000)
            } else {
                format!("{} MB", size / 1_000_000)
            };

            if !path.is_dir() {
                return Some(size_str);
            }
        }
        None
    }

    pub fn last_modified(path: &Path) -> Option<String> {
        if let Ok(metadata) = path.metadata() {
            let last_write_time = metadata.last_write_time();
            if let Some(date) = Ex::windows_date(last_write_time as i64) {
                return Some(date.format("%d/%m/%Y %H:%M").to_string());
            }
        }
        None
    }

    // 1601-01-01 is 11,644,473,600 seconds before Unix epoch.
    //https://github.com/oylenshpeegul/Epochs-rust/blob/master/src/lib.rs
    fn windows_date(x: i64) -> Option<NaiveDateTime> {
        let d = 10_000_000;
        let s = -11_644_473_600;
        let q = x / d;
        let n = ((x % d) * (1_000_000_000 / d)) as u32;
        let t = q.checked_add(s)?;
        NaiveDateTime::from_timestamp_opt(t, n)
    }

    pub fn open(&self, path: &Path) -> Result<(), String> {
        match open::that(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    pub fn rename(&self, new_name: &str, file: &Path) -> io::Result<()> {
        let mut new_path = file.to_path_buf();
        new_path.set_file_name(new_name);

        fs::rename(file, new_path)
    }

    pub fn delete(&self, file: &Path) -> Result<(), trash::Error> {
        trash::delete(file)
    }

    pub fn create_file(&self, path: &Path) -> io::Result<()> {
        fs::File::create(path)?;
        Ok(())
    }

    pub fn create_dir(&self, path: &Path) -> io::Result<()> {
        fs::create_dir(path)
    }

    pub fn reset(&mut self) {
        self.files = Vec::new();
    }
}
