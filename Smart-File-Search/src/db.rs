use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct DbHandler {
    conn: Mutex<Connection>,
}

impl DbHandler {
    pub fn new() -> anyhow::Result<Self> {
        let connection_result = Connection::open("my_files.db");

        let conn = match connection_result {
            Ok(c) => c,
            Err(e) => return Err(anyhow::Error::new(e)),
        };

        let create_table_sql = "
            CREATE TABLE IF NOT EXISTS files (
                path TEXT NOT NULL,
                vector TEXT NOT NULL
            )
        ";

        if let Err(e) = conn.execute(create_table_sql, []) {
            eprintln!("Could not create table: {}", e);
            return Err(anyhow::Error::new(e));
        }

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert(&self, paths: Vec<String>, vecs: Vec<Vec<f32>>) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM files", [])?;

        let mut stmt = conn.prepare("INSERT INTO files (path, vector) VALUES (?1, ?2)")?;

        for (i, path) in paths.iter().enumerate() {
            let vec = &vecs[i];
            let vec_json_result = serde_json::to_string(vec);

            if let Ok(vec_json) = vec_json_result {
                stmt.execute(params![path, vec_json])?;
            }
        }
        Ok(())
    }

    pub fn get_all_files(&self) -> anyhow::Result<Vec<(String, Vec<f32>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT path, vector FROM files")?;

        let rows_iterator = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let vec_str: String = row.get(1)?;
            Ok((path, vec_str))
        })?;

        let mut files: Vec<(String, Vec<f32>)> = Vec::new();

        for row_result in rows_iterator {
            if let Ok((path, vec_str)) = row_result {
                let vec_parsing_result: Result<Vec<f32>, _> = serde_json::from_str(&vec_str);

                if let Ok(vec) = vec_parsing_result {
                    files.push((path, vec));
                }
            }
        }

        Ok(files)
    }
}