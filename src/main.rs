#![allow(dead_code)]
use rusqlite::{Connection, Result as SqlResult, ffi::sqlite3_auto_extension, params};
use sqlite_vec::sqlite3_vec_init;
use std::{
    any::Any,
    error::Error,
    fmt::Debug,
    io::{self, Write, stdout},
    path::PathBuf,
    time::SystemTime,
};
use walkdir::{DirEntry, WalkDir};
use zerocopy::IntoBytes;

const EMBEDDING_DIM: usize = 384; // Standard for MiniLM models

struct FileMetadata {
    file_name: String,
    path: PathBuf,
    created: SystemTime,
    len: u64,
}

impl Debug for FileMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in \t{}", self.file_name, self.path.display())
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn setup_database(db: &Connection) -> SqlResult<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            file_name TEXT NOT NULL,
            path TEXT NOT NULL,
            created INTEGER,
            len INTEGER
        )",
        [],
    )?;

    db.execute(
        &format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS file_embeddings USING vec0(
                embedding float[{EMBEDDING_DIM}]
            )"
        ),
        [],
    )?;

    Ok(())
}

fn insert_file(
    conn: &Connection,
    id: i64,
    file: &FileMetadata,
    embedding: &[f32],
) -> SqlResult<()> {
    let created_ts = file
        .created
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO files (id, file_name, path, created, len) VALUES (?, ?, ?, ?, ?)",
        params![
            id,
            file.file_name,
            file.path.to_string_lossy(),
            created_ts,
            file.len as i64
        ],
    )?;

    conn.execute(
        "INSERT INTO file_embeddings (rowid, embedding) VALUES (?, ?)",
        params![id, embedding.as_bytes()],
    )?;

    Ok(())
}

fn search_similar_files(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> SqlResult<Vec<(String, String, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT 
            f.file_name,
            f.path,
            e.distance
         FROM file_embeddings e
         JOIN files f ON f.id = e.rowid
         WHERE embedding MATCH ?
           AND k = ?",
    )?;

    let results = stmt
        .query_map(params![query_embedding.as_bytes(), limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

fn find_important_files(path: PathBuf) -> Result<Vec<FileMetadata>, Box<dyn Error>> {
    let mut files = vec![];

    let walker = WalkDir::new(path).into_iter();
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
                path: entry.path().to_path_buf(),
                created: metadata.created()?,
                len: metadata.len(),
            });
            // println!("{:?}", files.last().unwrap());
        }
    }
    Ok(files)
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from("/home/atduyar/Downloads/");
    // let mut path = env::current_dir()?;
    // path.push("test_folder/");
    println!("Searching path: {}", path.display());

    let files = find_important_files(path)?;

    let db = {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let db = Connection::open("file_index.db")?;
        setup_database(&db)?;
        // clear old data
        db.execute("DELETE FROM files", [])?;
        db.execute("DELETE FROM file_embeddings", [])?;
        db
    };

    // Initialize embedding model
    let mut model = fastembed::TextEmbedding::try_new(Default::default())?;

    // Batch embed all filenames (much faster)
    let texts: Vec<&str> = files.iter().map(|f| f.file_name.as_str()).collect();
    println!(
        "[LOG]: Batch embedding started with model {:?}",
        model.type_id()
    );
    let embeddings = model.embed(texts, None)?;
    println!(
        "[LOG]: Batch embedding finished on {} files",
        embeddings.len()
    );

    // Insert into DB
    for (id, (file, embedding)) in files.iter().zip(embeddings.iter()).enumerate() {
        insert_file(&db, (id + 1) as i64, file, embedding)?;
        print!(
            "\r[LOG]: Indexing [{:<20}] {}/{}",
            '#'.to_string()
                .repeat(((id as f32) / (files.len() as f32) * 20.0).round() as usize),
            id + 1,
            files.len()
        );
        stdout().flush().expect("Failed to flush stdout");
    }
    println!("\n");

    loop {
        let mut input_line = String::new();
        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");

        let query_embedding = model.embed(vec![input_line], None)?;
        let results = search_similar_files(&db, &query_embedding[0], 5)?;

        if results.is_empty() {
            println!("   No results found\n");
        } else {
            for (name, path, distance) in results.iter().rev() {
                println!("{:},{},{}", distance, name, path);
            }
            println!();
        }
    }
    // // Demo searches
    // for query in [
    //     // "code",
    //     // "document",
    //     // "Lab pdf",
    //     // "api",
    //     "magic", "CV", "homework", "school", "movie",
    // ] {
    //     println!("Search: \"{}\"", query);
    //
    //     let query_embedding = model.embed(vec![query], None)?;
    //     let results = search_similar_files(&db, &query_embedding[0], 5)?;
    //
    //     if results.is_empty() {
    //         println!("   No results found\n");
    //     } else {
    //         for (name, path, distance) in results.iter().rev() {
    //             println!("   {:.4} \t| {} ({})", distance, name, path);
    //         }
    //         println!();
    //     }
    // }
    Ok(())
}
