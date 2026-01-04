use crate::models::{FileMetadata, EMBEDDING_DIM, SearchResult, DbStats};
use rusqlite::{Connection, Result as SqlResult, ffi::sqlite3_auto_extension, params};
use sqlite_vec::sqlite3_vec_init;
use std::sync::Once;
use zerocopy::IntoBytes;

static INIT_SQLITE_EXTENSIONS: Once = Once::new();

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> SqlResult<Self> {
        INIT_SQLITE_EXTENSIONS.call_once(|| unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        });
        let conn = Connection::open(path)?;
        Self::setup_database(&conn)?;
        Ok(Self { conn })
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

    pub fn clear(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM files", [])?;
        self.conn.execute("DELETE FROM file_embeddings", [])?;
        Ok(())
    }

    pub fn insert_file(&self, id: i64, file: &FileMetadata, embedding: &[f32]) -> SqlResult<()> {
        use std::time::SystemTime;
        let created_ts = file
            .created
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        self.conn.execute(
            "INSERT INTO files (id, file_name, path, created, len) VALUES (?, ?, ?, ?, ?)",
            params![
                id,
                file.file_name,
                file.path.to_string_lossy(),
                created_ts,
                file.len as i64
            ],
        )?;

        self.conn.execute(
            "INSERT INTO file_embeddings (rowid, embedding) VALUES (?, ?)",
            params![id, embedding.as_bytes()],
        )?;

        Ok(())
    }

    pub fn search_similar(&self, query_embedding: &[f32], limit: usize) -> SqlResult<Vec<SearchResult>> {
        let mut stmt = self.conn.prepare(
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

    pub fn get_stats(&self) -> Result<DbStats, Box<dyn std::error::Error>> {
        use std::fs;

        let (file_count, last_updated): (i64, Option<i64>) = self.conn.query_row(
            "SELECT COUNT(*), MAX(created) FROM files", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            }
        )?;

        let db_path = self.conn.path().unwrap();
        let db_size_bytes = fs::metadata(db_path)?.len();

        Ok(DbStats {
            db_path: db_path.to_string(),
            file_count,
            db_size_bytes,
            embedding_dim: EMBEDDING_DIM,
            last_updated,
        })
    }
}
