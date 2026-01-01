use crate::{walker, ai, db};
use std::time::Instant;
use std::io::{self, Write};

pub async fn update_index(ai: &ai::AiHandler, db: &db::DbHandler) -> anyhow::Result<()> {
    let start_time = Instant::now();

    let download_dir = dirs::download_dir().unwrap();
    let path = download_dir.to_str().unwrap().to_string();

    println!("Scanning folder: {}", path);

    let files = walker::get_file_list(&path);
    let total_files = files.len();
    println!("Found {} files. Preparing batch processing...", total_files);

    let mut file_names: Vec<String> = Vec::new();
    for file in files {
        let name_string = file.to_string_lossy().to_string();
        file_names.push(name_string);
    }

    println!("Sending to AI (Batch Mode)...");
    let vectors = ai.get_embeddings_batch(file_names.clone())?;

    println!("Saving to database...");
    db.insert(file_names, vectors)?;

    let duration = start_time.elapsed();
    println!("Done in {:.2?}! Indexed {} files.", duration, total_files);
    Ok(())
}

pub fn search_and_open(ai: &ai::AiHandler, db: &db::DbHandler, query_text: &str) -> anyhow::Result<()> {
    println!("--------------------------------");
    println!("Searching for: '{}'", query_text);

    let query_vec = ai.get_embedding(query_text)?;
    let all_files = db.get_all_files()?;

    let mut results: Vec<(f32, String)> = Vec::new();

    for (path, vec) in all_files {
        let score = ai.similarity_score(&query_vec, &vec);
        results.push((score, path));
    }

    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let mut top_results: Vec<&(f32, String)> = Vec::new();
    for item in results.iter().take(3) {
        top_results.push(item);
    }

    if top_results.is_empty() {
        println!("No matching files found.");
        return Ok(());
    }

    println!("\nTop 3 matches found:");
    for (i, (score, path)) in top_results.iter().enumerate() {
        println!("{}. [{:.2}] {}", i + 1, score, path);
    }

    print!("\nSelect file to open (1-3) or '0' to cancel: ");
    io::stdout().flush()?;

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    let selection = selection.trim();

    let parse_result = selection.parse::<usize>();

    match parse_result {
        Ok(index) => {
            if index > 0 && index <= top_results.len() {
                let selected_file = &top_results[index - 1].1;
                println!("Opening: {}", selected_file);
                opener::open(selected_file)?;
            } else if index == 0 {
                println!("Selection cancelled.");
            } else {
                println!("Invalid selection.");
            }
        },
        Err(_) => {
            println!("Invalid input. Please enter a number.");
        }
    }

    Ok(())
}