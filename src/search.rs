use crate::db;
use crate::fzf;

pub fn run_filename(topic: &str) -> anyhow::Result<()> {
    let results = db::search_by_name(topic)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names.", topic);
        return Ok(());
    }

    fzf::require_fzf()?;

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}", backend, section, name, display_name)
        })
        .collect();

    let header = format!("{:<20}\t{}", "BACKEND", "NAME");

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4"), &lines)?;

    Ok(())
}

pub fn run_keyword(keyword: &str) -> anyhow::Result<()> {
    let results = db::search_by_keyword(keyword)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names and descriptions.", keyword);
        return Ok(());
    }

    fzf::require_fzf()?;

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name, description)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}\t{}", backend, section, name, display_name, description)
        })
        .collect();

    let header = format!("{:<20}\t{:<30}\t{}", "BACKEND", "NAME", "DESCRIPTION");

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4,5"), &lines)?;

    Ok(())
}

pub fn run_all() -> anyhow::Result<()> {
    let results = db::list_all_topics()?;
    if results.is_empty() {
        println!("No man pages indexed. Try installing a backend first.");
        return Ok(());
    }

    fzf::require_fzf()?;

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name, description)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}\t{}", backend, section, name, display_name, description)
        })
        .collect();

    let header = format!("{:<20}\t{:<30}\t{}", "BACKEND", "NAME", "DESCRIPTION");

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4,5"), &lines)?;

    Ok(())
}