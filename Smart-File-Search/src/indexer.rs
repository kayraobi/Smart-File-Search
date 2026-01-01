use crate::{walker, ai, db};
use std::time::Instant;
use std::io::{self, Write};

pub async fn update_index(ai: &ai::AiHandler, db: &db::DbHandler) -> anyhow::Result<()> {
    let start_time = Instant::now();
    let path = dirs::download_dir().unwrap().to_str().unwrap().to_string();
    println!("Scanning folder: {}", path);

    let files = walker::get_file_list(&path);
    let total_files = files.len();
    println!("Found {} files. Preparing batch processing...", total_files);

    let file_names: Vec<String> = files.iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

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

    let mut results: Vec<(f32, String)> = all_files.into_iter().map(|(path, vec)| {
        let score = ai.similarity_score(&query_vec, &vec);
        (score, path)
    }).collect();

    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let top_results: Vec<&(f32, String)> = results.iter().take(3).collect();

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

    if let Ok(index) = selection.parse::<usize>() {
        if index > 0 && index <= top_results.len() {
            let selected_file = &top_results[index - 1].1;
            println!("Opening: {}", selected_file);
            opener::open(selected_file)?;
        } else if index == 0 {
            println!("Selection cancelled.");
        } else {
            println!("Invalid selection.");
        }
    } else {
        println!("Invalid input. Please enter a number.");
    }

    Ok(())
}