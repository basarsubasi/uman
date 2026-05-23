use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use rusqlite::Connection;

use crate::config::BackendDef;
use crate::paths;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY,
    backend TEXT NOT NULL,
    section INTEGER NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    format TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    last_updated TEXT NOT NULL,
    UNIQUE(backend, section, name)
);

CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
    name, content='pages', content_rowid='id'
);
";

static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    paths::ensure_dirs().expect("Failed to create uman directories");
    let db_path = paths::db_path();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL;").expect("Failed to set WAL mode");
    conn.execute_batch(SCHEMA).expect("Failed to initialize schema");
    Mutex::new(conn)
});

pub fn with_conn<F, T>(f: F) -> anyhow::Result<T>
where
    F: FnOnce(&Connection) -> anyhow::Result<T>,
{
    let conn = DB.lock().map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;
    f(&conn)
}

fn parse_section_and_name(file_name: &str) -> Option<(i32, String)> {
    let trimmed = file_name.trim_end_matches(".gz");
    let parts: Vec<&str> = trimmed.rsplitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }
    let section: i32 = parts[0].parse().ok()?;
    let name = parts[1].to_string();
    Some((section, name))
}

fn hash_file(path: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Some(format!("{:x}", hasher.finalize()))
}

fn collect_man_pages(dir: &Path) -> Vec<(i32, String, String, String)> {
    let mut pages = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pages.extend(collect_man_pages(&path));
            } else {
                let file_name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };
                if let Some((section, name)) = parse_section_and_name(&file_name) {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(hash) = hash_file(&path) {
                        pages.push((section, name, path_str, hash));
                    }
                }
            }
        }
    }
    pages
}

fn iso_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = civil_from_days(days_since_epoch as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn index_backend(backend: &BackendDef) -> anyhow::Result<()> {
    let backend_dir = paths::backend_dir(&backend.name);
    if !backend_dir.exists() {
        anyhow::bail!("backend directory not found: {:?}", backend_dir);
    }

    let pages = collect_man_pages(&backend_dir);
    let now = iso_now();

    with_conn(|conn| {
        for (section, name, path, hash) in &pages {
            conn.execute(
                "INSERT OR REPLACE INTO pages (backend, section, name, path, format, content_hash, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![backend.name, section, name, path, backend.format, hash, now],
            )?;
        }

        let backend_pages: Vec<(i32, String)> = pages.iter().map(|(s, n, _, _)| (*s, n.clone())).collect();

        let mut stmt = conn.prepare(
            "SELECT section, name FROM pages WHERE backend = ?1",
        )?;

        let existing: Vec<(i32, String)> = stmt
            .query_map(rusqlite::params![backend.name], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (section, name) in &existing {
            if !backend_pages.contains(&(*section, name.clone())) {
                conn.execute(
                    "DELETE FROM pages WHERE backend = ?1 AND section = ?2 AND name = ?3",
                    rusqlite::params![backend.name, section, name],
                )?;
            }
        }

        conn.execute(
            "INSERT INTO pages_fts(pages_fts) VALUES('rebuild')",
            [],
        )?;

        println!("Indexed {} pages for backend '{}'.", pages.len(), backend.name);
        Ok(())
    })
}

pub fn remove_backend_entries(backend_name: &str) -> anyhow::Result<()> {
    with_conn(|conn| {
        conn.execute("DELETE FROM pages WHERE backend = ?1", rusqlite::params![backend_name])?;
        conn.execute("INSERT INTO pages_fts(pages_fts) VALUES('rebuild')", [])?;
        Ok(())
    })
}

pub fn search(topic: &str) -> anyhow::Result<Vec<(String, i32, String)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT backend, section, name FROM pages WHERE name LIKE ?1 ORDER BY name",
        )?;

        let pattern = format!("%{topic}%");
        let results: Vec<(String, i32, String)> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    })
}