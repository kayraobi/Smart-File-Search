use std::fs;
use std::path::Path;

fn main() {
    recursive_file_search(Path::new(
        "/Users/kayra/Documents/GitHub/Smart-File-Search/test_folder",
    ));
}

fn recursive_file_search(path: &Path) {
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading {:?}: {}", path, e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error entry: {}", e);
                continue;
            }
        };

        let p = entry.path();
        println!("{:?}", p);

        if p.is_dir() {
            recursive_file_search(&p);
        }
    }
}
