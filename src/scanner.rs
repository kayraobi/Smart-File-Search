use crate::models::FileMetadata;
use std::{error::Error, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn find_important_files(path: PathBuf) -> Result<Vec<FileMetadata>, Box<dyn Error>> {
    let mut files = vec![];

    let walker = WalkDir::new(&path).into_iter();
    for entry in walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            files.push(FileMetadata {
                file_name: entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                path: entry.path().canonicalize().unwrap_or_else(|_| entry.path().to_path_buf()),
                created: metadata.created()?,
                len: metadata.len(),
            });
        }
    }
    Ok(files)
}
