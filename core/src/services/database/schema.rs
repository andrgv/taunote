use rusqlite::{Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};

// Returns the path to the local SQLite database
fn get_db_path(base_dir: &Path) -> PathBuf {
    let db_dir = base_dir.join("db");
    fs::create_dir_all(&db_dir).expect("Failed to create database directory");
    return db_dir.join("project.db");
}

pub fn init_db(base_dir: &Path) -> Result<()> {
    let db_path = get_db_path(base_dir);
    let conn = Connection::open(db_path)?;
    let schema = include_str!("../../assets/schema.sql");
    conn.execute_batch(schema)?;
    Ok(())
}