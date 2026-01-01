use walkdir::WalkDir;
use std::path::PathBuf;

pub fn get_file_list(root_path: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    let allowed_extensions: [&str; 17] = [
        "pdf", "docx", "doc", "txt", "md",
        "png", "jpg", "jpeg", "webp",
        "xlsx", "csv",
        "pptx",
        "rs", "py", "js", "html", "css"
    ];

    let ignored_folders: [&str; 9] = [
        "node_modules", "target", "venv", "bin", "obj",
        "Library", "Applications", "Windows", "Program Files"
    ];

    for entry_result in WalkDir::new(root_path) {
        match entry_result {
            Ok(entry) => {
                let file_name = entry.file_name().to_string_lossy();
                let path = entry.path();

                if file_name.starts_with('.') {
                    continue;
                }

                if entry.file_type().is_dir() {
                    if ignored_folders.contains(&file_name.as_ref()) {
                        continue;
                    }
                }

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