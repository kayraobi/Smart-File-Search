use walkdir::{DirEntry, WalkDir};
use std::path::PathBuf;

fn is_valid_folder(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();

    if name.starts_with('.') {
        return false;
    }

    let ignored_folders = [
        "node_modules",
        "target",
        "venv",
        "bin", "obj",
        "Library",
        "Applications"
    ];

    if entry.file_type().is_dir() {
        if ignored_folders.contains(&name.as_ref()) {
            return false;
        }
    }

    true
}

pub fn get_file_list(root_path: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let allowed_extensions = [
        "pdf", "docx", "doc", "txt", "md",
        "png", "jpg", "jpeg", "webp",
        "xlsx", "csv",
        "pptx",
        "rs", "py", "js", "html", "css"
    ];

    for entry in WalkDir::new(root_path).into_iter().filter_entry(is_valid_folder) {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    if let Some(ext) = e.path().extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if allowed_extensions.contains(&ext_str.as_str()) {
                            files.push(e.into_path());
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }
    files
}