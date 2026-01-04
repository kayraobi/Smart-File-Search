use std::env;
use std::io::{self, stderr};
use std::path::PathBuf;

mod context;
mod db;
mod embedding;
mod models;
mod scanner;

use context::AppContext;

fn eprint_info(msg: &str) {
    use std::io::Write;
    writeln!(stderr(), "[INFO] {}", msg).ok();
}

fn eprint_log(msg: &str) {
    use std::io::Write;
    writeln!(stderr(), "[LOG] {}", msg).ok();
}

fn eprint_error(msg: &str) {
    use std::io::Write;
    writeln!(stderr(), "[ERROR] {}", msg).ok();
}

fn parse_path(args: &[String]) -> PathBuf {
    let default_path = String::from(".");
    let path_str = args
        .iter()
        .position(|a| a == "-p")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&default_path);

    let mut path = if path_str == "." {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(path_str)
    };

    if path_str.starts_with("./") {
        if let Ok(cwd) = env::current_dir() {
            path = cwd.join(path_str.trim_start_matches("./"));
        }
    }

    path
}

fn info_command() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AppContext::new()?;
    let stats = ctx.db().get_stats()?;

    eprint_info(&format!("Database location: {}", stats.db_path));
    eprint_info(&format!("Files indexed: {}", stats.file_count));
    eprint_info(&format!("Database size: {} bytes", stats.db_size_bytes));
    eprint_info(&format!("Embedding dimension: {}", stats.embedding_dim));

    match stats.last_updated {
        Some(ts) => eprint_info(&format!("Last updated: {}", ts)),
        None => eprint_info("Last updated: Never"),
    }

    Ok(())
}

fn update_db(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    eprint_info(&format!("Searching path: {}", path.display()));

    let files = scanner::find_important_files(path)?;

    let db_path = env::temp_dir().join("file_index.db");
    eprint_info(&format!("Database: {}", db_path.display()));

    let ctx = AppContext::new()?;
    ctx.db().clear()?;

    let texts: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();
    eprint_log("Batch embedding started");

    let mut ctx = ctx;
    let embeddings = ctx.model_mut().embed_batch(texts)?;
    eprint_log(&format!("Batch embedding finished on {} files", embeddings.len()));

    for (id, (file, embedding)) in files.iter().zip(embeddings.iter()).enumerate() {
        ctx.db().insert_file((id + 1) as i64, file, embedding)?;
        write!(
            stderr(),
            "\r[LOG] Indexing [{:<20}] {}/{}",
            '#'.to_string()
                .repeat(((id as f32) / (files.len() as f32) * 20.0).round() as usize),
            id + 1,
            files.len()
        )?;
        stderr().flush()?;
    }
    writeln!(stderr())?;
    eprint_info("Indexing complete!");

    info_command()?;

    Ok(())
}

fn search(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = AppContext::new()?;
    let output = ctx.search(query)?;
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn interactive_search() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = AppContext::new()?;
    eprint_info("Interactive search mode (Ctrl+C to exit)");

    loop {
        let mut input_line = String::new();
        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");

        let query = input_line.trim();
        if query.is_empty() {
            continue;
        }

        let output = ctx.search(query)?;
        println!("{}", serde_json::to_string(&output)?);
    }
}

fn print_help() {
    println!("Usage: <command> [options]");
    println!("\nCommands:");
    println!("  update-db [-p <path>]    - Update database with files");
    println!("  search <query>            - Search files");
    println!("  interactive-search        - Interactive search mode");
    println!("  info                      - Show database statistics");
    println!("\nOptions:");
    println!("  -p <path>                 - Path to search (default: current directory)");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "update-db" => {
            let path = parse_path(&args);
            update_db(path)?;
        }
        "search" => {
            if args.len() < 3 {
                eprint_error("Usage: search <query>");
                return Ok(());
            }
            search(&args[2])?;
        }
        "interactive-search" => {
            interactive_search()?;
        }
        "info" => {
            info_command()?;
        }
        _ => {
            print_help();
        }
    }

    Ok(())
}
