use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct DbHandler {
    conn: Mutex<Connection>,
}

impl DbHandler {
    pub fn new() -> anyhow::Result<Self> {
        let conn = Connection::open("my_files.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT NOT NULL,
                vector TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert(&self, paths: Vec<String>, vecs: Vec<Vec<f32>>) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM files", [])?;

        let mut stmt = conn.prepare("INSERT INTO files (path, vector) VALUES (?1, ?2)")?;

        for (path, vec) in paths.iter().zip(vecs.iter()) {
            let vec_json = serde_json::to_string(vec)?;
            stmt.execute(params![path, vec_json])?;
        }
        Ok(())
    }

    pub fn get_all_files(&self) -> anyhow::Result<Vec<(String, Vec<f32>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT path, vector FROM files")?;

        let files = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let vec_str: String = row.get(1)?;
            let vec: Vec<f32> = serde_json::from_str(&vec_str).unwrap();
            Ok((path, vec))
        })?
            .filter_map(Result::ok)
            .collect();

        Ok(files)
    }
}