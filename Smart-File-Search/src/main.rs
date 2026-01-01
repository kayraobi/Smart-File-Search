mod walker;
mod indexer;
mod ai;
mod db;

use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ai = ai::AiHandler::new()?;
    let db = db::DbHandler::new()?;

    indexer::update_index(&ai, &db).await?;

    loop {
        println!("\n--------------------------------");
        print!("Search query (or type 'exit' to quit): ");
        io::stdout().flush()?;

        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        let query = query.trim();

        if query == "exit" {
            break;
        }

        if !query.is_empty() {
            indexer::search_and_open(&ai, &db, query)?;
        }
    }

    Ok(())
}