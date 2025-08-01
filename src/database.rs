use std::env;

use anyhow::{Context, Result};
use rusqlite::Connection;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Error while opening sqlite file {path}"))?;
        let s = Self { conn };
        s.init_tables().context("Error while creating SQL tables")?;
        Ok(s)
    }

    pub fn by_env() -> Result<Self> {
        Self::new(
            env::var("SQLITE_PATH")
                .unwrap_or_else(|_| "database.sqlite".to_string())
                .as_str(),
        )
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(include_str!("tables.sql"))?;
        Ok(())
    }
}
