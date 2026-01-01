#![allow(dead_code)]
use std::{env, error::Error, fmt::Debug, path::PathBuf, time::SystemTime};
use walkdir::{DirEntry, WalkDir};

struct FileMetadate {
    file_name: String,
    path: PathBuf,
    created: SystemTime,
    len: u64
}

impl Debug for FileMetadate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in \t{}", self.file_name, self.path.display())
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = vec![];
    let path = env::current_dir()?;
    println!("Searching path: {}", path.display());

    let walker = WalkDir::new(path).into_iter();
    for entry in walker
        .filter_entry(|e| !is_hidden(e) )
        .filter_map(|e| e.ok())
    {
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            db.push(FileMetadate {
                file_name: entry.path().file_name().unwrap().to_string_lossy().into_owned(),
                path: entry.path().to_path_buf(),
                created: metadata.created()?,
                len: metadata.len(),
            });
            println!("{:?}", db.last().unwrap());
        }
    }
    Ok(())
}

