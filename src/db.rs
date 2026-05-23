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
    description TEXT NOT NULL DEFAULT '',
    path TEXT NOT NULL,
    format TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    last_updated TEXT NOT NULL,
    UNIQUE(backend, section, name)
);
";

const SCHEMA_V2_FTS: &str = "
DROP TABLE IF EXISTS pages_fts;
CREATE VIRTUAL TABLE pages_fts USING fts5(
    name, description, content='pages', content_rowid='id'
);
";

static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    paths::ensure_dirs().expect("Failed to create uniman directories");
    let db_path = paths::db_path();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL;").expect("Failed to set WAL mode");
    conn.execute_batch(SCHEMA).expect("Failed to initialize schema");
    migrate_schema(&conn).expect("Failed to migrate schema");
    Mutex::new(conn)
});

fn migrate_schema(conn: &Connection) -> anyhow::Result<()> {
    // Add description column if it doesn't exist (migration from v1 to v2)
    let has_description: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM pragma_table_info('pages') WHERE name='description'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_description {
        conn.execute_batch("ALTER TABLE pages ADD COLUMN description TEXT NOT NULL DEFAULT '';")?;
    }

    // Recreate FTS table with the new schema (safe to rerun)
    conn.execute_batch(SCHEMA_V2_FTS)?;
    conn.execute("INSERT INTO pages_fts(pages_fts) VALUES('rebuild')", [])?;
    Ok(())
}

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

fn extract_description(file_path: &Path) -> String {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let mut in_name_section = false;
    let mut description_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(".SH") || trimmed.starts_with(".Sh") {
            let section_name = trimmed[3..].trim().to_lowercase();
            if section_name == "name" {
                in_name_section = true;
                continue;
            } else if in_name_section {
                break;
            }
        }

        if in_name_section {
            // Skip empty lines and roff commands
            if trimmed.is_empty() || trimmed.starts_with('.') {
                continue;
            }

            // This is a line in the NAME section
            description_lines.push(trimmed.to_string());
        }
    }

    if description_lines.is_empty() {
        return String::new();
    }

    let raw = description_lines.join(" ");

    // Clean up roff formatting
    let cleaned = raw
        .replace("\\- ", "- ")
        .replace("\\-", "-")
        .replace("\\fB", "")
        .replace("\\fI", "")
        .replace("\\fP", "")
        .replace("\\fR", "")
        .replace("\\&", "")
        .replace("\\(tm", "TM")
        .replace("\\(reg", "(R)")
        .replace("  ", " ");

    // Strip the name portion before " - " to leave just the description
    if let Some(idx) = cleaned.find(" - ") {
        cleaned[idx + 3..].trim().to_string()
    } else {
        cleaned.trim().to_string()
    }
}

fn collect_man_pages(dir: &Path) -> Vec<(i32, String, String, String, String)> {
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
                        let description = extract_description(&path);
                        pages.push((section, name, path_str, hash, description));
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
    let format_str = backend.format.to_string();

    with_conn(|conn| {
        let fs_pages: Vec<(i32, String, String)> = pages
            .iter()
            .map(|(s, n, _, h, _)| (*s, n.clone(), h.clone()))
            .collect();

        let mut stmt = conn.prepare(
            "SELECT section, name, content_hash FROM pages WHERE backend = ?1",
        )?;

        let existing: Vec<(i32, String, String)> = stmt
            .query_map(rusqlite::params![backend.name], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let existing_map: std::collections::HashMap<(i32, String), String> = existing
            .iter()
            .map(|(s, n, h)| ((*s, n.clone()), h.clone()))
            .collect();

        let fs_map: std::collections::HashMap<(i32, String), String> = fs_pages
            .iter()
            .map(|(s, n, h)| ((*s, n.clone()), h.clone()))
            .collect();

        let mut inserted = 0;
        let mut updated = 0;
        let mut deleted = 0;

        for (section, name, path, hash, description) in &pages {
            let key = (*section, name.clone());
            match existing_map.get(&key) {
                Some(existing_hash) if existing_hash == hash => {
                    // Unchanged — but description might have been added in v2 migration
                    // Check if description needs updating
                    let needs_desc_update: bool = conn
                        .query_row(
                            "SELECT length(description) = 0 FROM pages WHERE backend = ?1 AND section = ?2 AND name = ?3",
                            rusqlite::params![backend.name, section, name],
                            |row| row.get(0),
                        )
                        .unwrap_or(true);

                    if needs_desc_update {
                        conn.execute(
                            "UPDATE pages SET description = ?1, last_updated = ?2 WHERE backend = ?3 AND section = ?4 AND name = ?5",
                            rusqlite::params![description, now, backend.name, section, name],
                        )?;
                        updated += 1;
                    }
                }
                _ => {
                    conn.execute(
                        "INSERT OR REPLACE INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        rusqlite::params![backend.name, section, name, path, format_str, hash, description, now],
                    )?;
                    if existing_map.contains_key(&key) {
                        updated += 1;
                    } else {
                        inserted += 1;
                    }
                }
            }
        }

        for (section, name, _) in &existing {
            let key = (*section, name.clone());
            if !fs_map.contains_key(&key) {
                conn.execute(
                    "DELETE FROM pages WHERE backend = ?1 AND section = ?2 AND name = ?3",
                    rusqlite::params![backend.name, section, name],
                )?;
                deleted += 1;
            }
        }

        if inserted > 0 || updated > 0 || deleted > 0 {
            conn.execute(
                "INSERT INTO pages_fts(pages_fts) VALUES('rebuild')",
                [],
            )?;
        }

        println!(
            "Indexed backend '{}': {} added, {} updated, {} removed, {} unchanged (total {})",
            backend.name,
            inserted,
            updated,
            deleted,
            fs_pages.len() - inserted - updated,
            fs_pages.len()
        );
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

pub fn search_by_name(topic: &str) -> anyhow::Result<Vec<(String, i32, String)>> {
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

pub fn search_by_keyword(keyword: &str) -> anyhow::Result<Vec<(String, i32, String, String)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT p.backend, p.section, p.name, p.description
             FROM pages p
             JOIN pages_fts f ON f.rowid = p.id
             WHERE pages_fts MATCH ?1
             ORDER BY rank",
        )?;

        let pattern = keyword.to_string();
        let results: Vec<(String, i32, String, String)> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    })
}

pub fn list_all_topics() -> anyhow::Result<Vec<(String, i32, String, String)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT backend, section, name, description FROM pages ORDER BY backend ASC, section ASC, name ASC",
        )?;

        let results: Vec<(String, i32, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    })
}

pub fn list_topics_for_backend(backend: &str) -> anyhow::Result<Vec<(i32, String, String)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT section, name, description FROM pages WHERE backend = ?1 ORDER BY section ASC, name ASC",
        )?;

        let results: Vec<(i32, String, String)> = stmt
            .query_map(rusqlite::params![backend], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    })
}

pub fn find_page(backend: &str, topic: &str) -> anyhow::Result<Option<(i32, String)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT section, name FROM pages WHERE backend = ?1 AND name = ?2 ORDER BY section ASC",
        )?;

        let results: Vec<(i32, String)> = stmt
            .query_map(rusqlite::params![backend, topic], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results.into_iter().next())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_man_page_name() {
        assert_eq!(
            parse_section_and_name("execve.2"),
            Some((2, "execve".to_string()))
        );
    }

    #[test]
    fn parse_man_page_with_multiple_dots() {
        assert_eq!(
            parse_section_and_name("foo.bar.3"),
            Some((3, "foo.bar".to_string()))
        );
    }

    #[test]
    fn parse_gzipped_man_page() {
        assert_eq!(
            parse_section_and_name("printf.3.gz"),
            Some((3, "printf".to_string()))
        );
    }

    #[test]
    fn parse_multi_dot_gzipped() {
        assert_eq!(
            parse_section_and_name("foo.bar.1.gz"),
            Some((1, "foo.bar".to_string()))
        );
    }

    #[test]
    fn reject_no_extension() {
        assert_eq!(parse_section_and_name("Makefile"), None);
    }

    #[test]
    fn reject_non_numeric_extension() {
        assert_eq!(parse_section_and_name("readme.txt"), None);
    }

    #[test]
    fn reject_empty_string() {
        assert_eq!(parse_section_and_name(""), None);
    }

    #[test]
    fn parse_section_1() {
        assert_eq!(parse_section_and_name("ls.1"), Some((1, "ls".to_string())));
    }

    #[test]
    fn parse_section_8() {
        assert_eq!(
            parse_section_and_name("mount.8"),
            Some((8, "mount".to_string()))
        );
    }

    #[test]
    fn parse_single_char_name() {
        assert_eq!(parse_section_and_name("X.7"), Some((7, "X".to_string())));
    }

    #[test]
    fn parse_underscore_name() {
        assert_eq!(
            parse_section_and_name("__libc_start_main.3"),
            Some((3, "__libc_start_main".to_string()))
        );
    }

    // --- extract_description ---

    #[test]
    fn extract_description_basic() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("execve.2");
        std::fs::write(&file, ".TH EXECVE 2\n.SH NAME\nexecve \\- execute program\n.SH SYNOPSIS\n").unwrap();
        let desc = extract_description(&file);
        assert_eq!(desc, "execute program");
    }

    #[test]
    fn extract_description_strips_roff() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("open.2");
        std::fs::write(&file, ".SH NAME\n\\fBopen\\fP, \\fBopenat\\fP \\- open and possibly create a file\n.SH SYNOPSIS\n").unwrap();
        let desc = extract_description(&file);
        assert!(desc.contains("open and possibly create a file"));
        assert!(!desc.contains("\\fB"));
        assert!(!desc.contains("open, openat"));
    }

    #[test]
    fn extract_description_multiline() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("printf.3");
        std::fs::write(&file, ".SH NAME\nprintf, fprintf, sprintf \\- formatted output conversion\n").unwrap();
        let desc = extract_description(&file);
        assert!(desc.contains("formatted output conversion"));
    }

    #[test]
    fn extract_description_no_name_section() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("unknown.1");
        std::fs::write(&file, ".SH SYNOPSIS\nSome stuff\n").unwrap();
        let desc = extract_description(&file);
        assert!(desc.is_empty());
    }

    #[test]
    fn extract_description_dash_handled() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ls.1");
        std::fs::write(&file, ".SH NAME\nls \\- list directory contents\n").unwrap();
        let desc = extract_description(&file);
        assert_eq!(desc, "list directory contents");
    }

    // --- civil_from_days ---

    #[test]
    fn epoch_day_zero_is_1970_01_01() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn known_date_2024_march_1() {
        assert_eq!(civil_from_days(19783), (2024, 3, 1));
    }

    #[test]
    fn known_date_2000_01_01() {
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
    }

    #[test]
    fn known_date_1999_12_31() {
        assert_eq!(civil_from_days(10956), (1999, 12, 31));
    }

    #[test]
    fn leap_day_2024_feb_29() {
        assert_eq!(civil_from_days(19782), (2024, 2, 29));
    }

    #[test]
    fn negative_days_1969_12_31() {
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn far_future_date() {
        assert_eq!(civil_from_days(47482), (2100, 1, 1));
    }

    // --- hash_file ---

    #[test]
    fn hash_file_produces_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.1");
        std::fs::write(&file_path, "hello world").unwrap();
        let hash = hash_file(&file_path);
        assert!(hash.is_some());
        let h = hash.unwrap();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_file_nonexistent_returns_none() {
        let hash = hash_file(Path::new("/nonexistent/path/file.1"));
        assert!(hash.is_none());
    }

    #[test]
    fn hash_file_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.1");
        std::fs::write(&file_path, "consistent content").unwrap();
        let h1 = hash_file(&file_path);
        let h2 = hash_file(&file_path);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_file_differs_for_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = dir.path().join("a.1");
        let f2 = dir.path().join("b.1");
        std::fs::write(&f1, "content A").unwrap();
        std::fs::write(&f2, "content B").unwrap();
        assert_ne!(hash_file(&f1), hash_file(&f2));
    }

    // --- collect_man_pages ---

    #[test]
    fn collect_from_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let pages = collect_man_pages(dir.path());
        assert!(pages.is_empty());
    }

    #[test]
    fn collect_finds_man_pages() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("execve.2"), "man page content").unwrap();
        std::fs::write(dir.path().join("printf.3"), "another page").unwrap();

        let pages = collect_man_pages(dir.path());
        assert_eq!(pages.len(), 2);

        let names: Vec<&str> = pages.iter().map(|(_, n, _, _, _)| n.as_str()).collect();
        assert!(names.contains(&"execve"));
        assert!(names.contains(&"printf"));
    }

    #[test]
    fn collect_skips_non_man_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Makefile"), "build stuff").unwrap();
        std::fs::write(dir.path().join("README.txt"), "read this").unwrap();
        std::fs::write(dir.path().join("execve.2"), "man page").unwrap();

        let pages = collect_man_pages(dir.path());
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].1, "execve");
    }

    #[test]
    fn collect_recurses_into_subdirs() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("man2");
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(subdir.join("open.2"), "open page").unwrap();

        let pages = collect_man_pages(dir.path());
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].1, "open");
    }

    #[test]
    fn collect_handles_gzipped_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("read.2.gz"), "compressed").unwrap();

        let pages = collect_man_pages(dir.path());
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].1, "read");
        assert_eq!(pages[0].0, 2);
    }

    #[test]
    fn collect_extracts_description() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("execve.2"),
            ".SH NAME\nexecve \\- execute program\n",
        )
        .unwrap();

        let pages = collect_man_pages(dir.path());
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].4, "execute program");
    }

    // --- Database operations with test DB ---

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn.execute_batch(SCHEMA_V2_FTS).unwrap();
        conn
    }

    #[test]
    fn db_schema_creates_tables() {
        let conn = create_test_db();

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='pages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='pages_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn db_insert_and_query() {
        let conn = create_test_db();

        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                "test-backend", 2, "execve", "/path/to/execve.2", "roff", "abc123", "execute program", "2024-01-01T00:00:00Z"
            ],
        ).unwrap();

        let name: String = conn
            .query_row("SELECT name FROM pages WHERE backend = 'test-backend'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "execve");

        let desc: String = conn
            .query_row("SELECT description FROM pages WHERE backend = 'test-backend'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(desc, "execute program");
    }

    #[test]
    fn db_fts_keyword_search() {
        let conn = create_test_db();

        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["test", 2, "execve", "/execve.2", "roff", "h", "execute a program", "2024-01-01T00:00:00Z"],
        ).unwrap();
        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["test", 2, "open", "/open.2", "roff", "h", "open and possibly create a file", "2024-01-01T00:00:00Z"],
        ).unwrap();
        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["test", 3, "printf", "/printf.3", "roff", "h", "formatted output conversion", "2024-01-01T00:00:00Z"],
        ).unwrap();

        conn.execute("INSERT INTO pages_fts(pages_fts) VALUES('rebuild')", []).unwrap();

        // Search for "execute" should find execve
        let mut stmt = conn.prepare(
            "SELECT p.name FROM pages p JOIN pages_fts f ON f.rowid = p.id WHERE pages_fts MATCH ?1 ORDER BY rank",
        ).unwrap();
        let results: Vec<String> = stmt
            .query_map(rusqlite::params!["execute"], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(results, vec!["execve"]);

        // Search for "file" should find open (in description) and printf (no, "file" not in printf's desc)
        let mut stmt = conn.prepare(
            "SELECT p.name FROM pages p JOIN pages_fts f ON f.rowid = p.id WHERE pages_fts MATCH ?1 ORDER BY rank",
        ).unwrap();
        let results: Vec<String> = stmt
            .query_map(rusqlite::params!["file"], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(results.contains(&"open".to_string()));
    }

    #[test]
    fn db_unique_constraint() {
        let conn = create_test_db();

        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                "test", 2, "dup", "/p", "roff", "h1", "", "2024-01-01T00:00:00Z"
            ],
        ).unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                "test", 2, "dup", "/p2", "roff", "h2", "updated desc", "2024-01-02T00:00:00Z"
            ],
        ).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM pages WHERE backend = 'test'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let path: String = conn
            .query_row("SELECT description FROM pages WHERE backend = 'test'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(path, "updated desc");
    }

    #[test]
    fn db_delete_backend() {
        let conn = create_test_db();

        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["backend-a", 2, "foo", "/a", "roff", "h", "desc a", "2024-01-01T00:00:00Z"],
        ).unwrap();
        conn.execute(
            "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["backend-b", 3, "bar", "/b", "roff", "h", "desc b", "2024-01-01T00:00:00Z"],
        ).unwrap();

        conn.execute("DELETE FROM pages WHERE backend = 'backend-a'", []).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM pages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let name: String = conn
            .query_row("SELECT name FROM pages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "bar");
    }

    #[test]
    fn db_like_search() {
        let conn = create_test_db();

        for (name, section) in [("execve", 2), ("execveat", 2), ("fexecve", 3), ("open", 2)] {
            conn.execute(
                "INSERT INTO pages (backend, section, name, path, format, content_hash, description, last_updated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    "test", section, name, format!("/{}", name), "roff", "h", "", "2024-01-01T00:00:00Z"
                ],
            ).unwrap();
        }

        let mut stmt = conn
            .prepare("SELECT name FROM pages WHERE name LIKE ?1 ORDER BY name")
            .unwrap();
        let results: Vec<String> = stmt
            .query_map(rusqlite::params!["%execve%"], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(results, vec!["execve", "execveat", "fexecve"]);
    }

    // --- iso_now format ---

    #[test]
    fn iso_now_produces_valid_format() {
        let now = iso_now();
        assert!(now.contains('T'), "iso_now should contain 'T': {now}");
        assert!(now.ends_with('Z'), "iso_now should end with 'Z': {now}");
        assert_eq!(now.len(), 20, "iso_now should be 20 chars: {now}");
    }
}