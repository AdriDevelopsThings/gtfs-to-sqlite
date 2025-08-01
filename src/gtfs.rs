use std::{collections::HashMap, env, fs::File};

use anyhow::{Context, Result, anyhow};
use rusqlite::ToSql;
use zip::{ZipArchive, read::ZipFile};

use crate::{csv_reader::CsvReader, database::Database};

pub struct GtfsFile {
    archive: ZipArchive<File>,
}

impl GtfsFile {
    pub fn new(file: File) -> Result<Self> {
        let archive = ZipArchive::new(file).context("Error while reading zip file")?;

        Ok(Self { archive })
    }

    pub fn by_env() -> Result<Self> {
        let path = env::var("GTFS_FILE")
            .map_err(|_| anyhow!("Environment variable 'GTFS_FILE' not set"))?;
        let file = File::open(&path).with_context(|| format!("Error while opening file {path}"))?;
        Self::new(file)
    }

    pub fn open_file(&mut self, name: &str) -> Result<CsvReader<ZipFile<File>>> {
        let file = self.archive.by_name(name).with_context(|| {
            format!("Error while reading file {name} from zip file, maybe it's not existing?")
        })?;
        let size = file.size() as usize;
        CsvReader::new(file, size)
    }

    pub fn import(
        &mut self,
        database: &mut Database,
        file_name: &str,
        table_name: &str,
    ) -> Result<()> {
        println!("Loading {table_name}...");

        let conn = &mut database.conn;
        let transaction = conn.transaction()?;
        if transaction
            .query_one(
                "SELECT name FROM already_loaded WHERE name=?",
                [table_name],
                |r| r.get::<_, String>(0),
            )
            .is_ok()
        {
            println!("Skipping loading {table_name} because it's already loaded");
            return Ok(());
        };

        transaction.execute(&format!("DELETE FROM {table_name}"), ())?;

        if table_name == "stops" {
            transaction.execute("PRAGMA defer_foreign_keys = ON", ())?;
        }

        let mut csv = self.open_file(file_name)?;
        let headers = csv.get_headers().to_vec();

        let mut stmt = transaction.prepare(&format!("PRAGMA table_info({table_name})"))?;
        let rows = stmt
            .query_map((), |row| {
                Ok((row.get::<_, String>("name")?, row.get::<_, String>("type")?))
            })?
            .collect::<Result<Vec<(String, String)>, _>>()?
            .into_iter()
            .filter(|r| headers.contains(&r.0))
            .collect::<Vec<_>>();
        let map: HashMap<String, String> = rows.into_iter().collect();
        drop(stmt);

        if map.len() != headers.len() {
            return Err(anyhow!(
                "Rows in sqlite are: {map:?} but csv headers are {headers:?}"
            ));
        }

        let query = format!(
            "INSERT INTO {table_name}({}) VALUES ({})",
            headers.join(","),
            headers
                .iter()
                .map(|_| "?".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut stmt = transaction.prepare(&query)?;
        let mut o_row = csv.read_row()?;
        let mut rows: usize = 0;
        while let Some(row) = o_row {
            let params =
                row.row
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let key = &headers[i];
                        let sql_type = map.get(key).unwrap();
                        let b: Box<dyn ToSql> =
                            if s.is_empty() {
                                Box::new(None::<&str>)
                            } else {
                                match sql_type.to_ascii_lowercase().as_str() {
                                    "text" => Box::new(s.as_str()),
                                    "integer" => Box::new(s.parse::<u64>().with_context(|| {
                                        format!("Invalid integer {s} in row {rows}")
                                    })?),
                                    "real" => Box::new(s.parse::<f64>().with_context(|| {
                                        format!("Invalid float {s} in row {rows}")
                                    })?),
                                    _ => panic!("{sql_type} is invalid sql type"),
                                }
                            };

                        Ok(b)
                    })
                    .collect::<Result<Vec<_>>>()?;

            stmt.execute(rusqlite::params_from_iter(params))
                .context("Error while inserting new stop")?;

            rows += 1;

            if rows % 16384 == 0 {
                println!("[{table_name}] {:.2}% ({rows})", (csv.processed() * 100f32));
            }
            o_row = csv.read_row()?;
        }
        drop(stmt);
        transaction.execute("INSERT INTO already_loaded(name) VALUES(?)", [table_name])?;

        println!("Loaded {table_name}");
        transaction.commit()?;

        Ok(())
    }
}
