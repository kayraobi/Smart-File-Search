use walkdir::{DirEntry, WalkDir};
use std::path::PathBuf;

fn is_ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();

    if name.starts_with('.') {
        return true;
    }

    let ignored_folders = [
        "node_modules", "target", "venv", "bin", "obj",
        "Library", "Applications", "Windows", "Program Files",
        ".git", ".vscode", ".idea"
    ];

    if entry.file_type().is_dir() {
        if ignored_folders.contains(&name.as_ref()) {
            return true;
        }
    }

    return false;
}

pub fn get_file_list(root_path: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    let allowed_extensions: [&str; 17] = [
        "pdf", "docx", "doc", "txt", "md",
        "png", "jpg", "jpeg", "webp",
        "xlsx", "csv",
        "pptx",
        "rs", "py", "js", "html", "css"
    ];

    let walker = WalkDir::new(root_path).into_iter().filter_entry(|e| !is_ignored(e));

    for entry_result in walker {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();

                if entry.file_type().is_file() {
                    match path.extension() {
                        Some(ext) => {
                            let ext_str = ext.to_string_lossy().to_lowercase();

                            if allowed_extensions.contains(&ext_str.as_str()) {
                                files.push(entry.into_path());
                            }
                        },
                        None => {
                            continue;
                        }
                    }
                }
            },
            Err(error) => {
                eprintln!("Error accessing file: {}", error);
                continue;
            }
        }
    }

    return files;
}