use std::path::Path;

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
    name, content, content='pages', content_rowid='id'
);
";

pub fn open() -> anyhow::Result<Connection> {
    paths::ensure_dirs()?;
    let db_path = paths::db_path();
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
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

fn collect_man_pages(
    dir: &Path,
    backend_name: &str,
) -> Vec<(i32, String, String, String)> {
    let mut pages = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pages.extend(collect_man_pages(&path, backend_name));
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

pub fn index_backend(backend: &BackendDef) -> anyhow::Result<()> {
    let backend_dir = paths::backend_dir(&backend.name);
    if !backend_dir.exists() {
        anyhow::bail!("backend directory not found: {:?}", backend_dir);
    }

    let conn = open()?;
    let pages = collect_man_pages(&backend_dir, &backend.name);
    let now = chrono_now();

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

    println!("Indexed {} pages for backend '{}'.", pages.len(), backend.name);
    Ok(())
}

pub fn remove_backend_entries(backend_name: &str) -> anyhow::Result<()> {
    let conn = open()?;
    conn.execute("DELETE FROM pages WHERE backend = ?1", rusqlite::params![backend_name])?;
    Ok(())
}

pub fn search(topic: &str) -> anyhow::Result<Vec<(String, i32, String)>> {
    let conn = open()?;
    let mut stmt = conn.prepare(
        "SELECT backend, section, name FROM pages WHERE name LIKE ?1",
    )?;

    let pattern = format!("%{topic}%");
    let results = stmt
        .query_map(rusqlite::params![pattern], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}