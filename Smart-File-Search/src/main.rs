mod walker;
mod indexer;
mod ai;
mod db;

use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ai_result = ai::AiHandler::new();
    let ai = match ai_result {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("Failed to initialize AI: {}", e);
            return Err(e);
        }
    };

    let db_result = db::DbHandler::new();
    let db = match db_result {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            return Err(e);
        }
    };

    if let Err(e) = indexer::update_index(&ai, &db).await {
        eprintln!("Error during indexing: {}", e);
    }

    loop {
        println!("\n--------------------------------");
        print!("Search query (or type 'exit' to quit): ");

        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
        }

        let mut query = String::new();
        match io::stdin().read_line(&mut query) {
            Ok(_) => {
                let query = query.trim();

                if query == "exit" {
                    println!("Goodbye!");
                    break;
                }

                if !query.is_empty() {
                    if let Err(e) = indexer::search_and_open(&ai, &db, query) {
                        eprintln!("Error during search: {}", e);
                    }
                }
            },
            Err(error) => {
                eprintln!("Failed to read input: {}", error);
            }
        }
    }

    Ok(())
}