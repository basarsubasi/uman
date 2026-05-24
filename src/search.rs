use crate::db;
use crate::fzf;

pub fn run_filename(topic: &str, plain_text: bool) -> anyhow::Result<()> {
    let results = db::search_by_name(topic)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names.", topic);
        return Ok(());
    }

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}", backend, section, name, display_name)
        })
        .collect();

    let header = format!("{:<20}\t{}", "BACKEND", "NAME");

    if plain_text {
        println!("{}", header);
        for line in &lines {
            println!("{}", line);
        }
        return Ok(());
    }

    fzf::require_fzf()?;

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4"), &lines)?;

    Ok(())
}

pub fn run_keyword(keyword: &str, plain_text: bool) -> anyhow::Result<()> {
    let results = db::search_by_keyword(keyword)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names and descriptions.", keyword);
        return Ok(());
    }

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name, description)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}\t{}", backend, section, name, display_name, description)
        })
        .collect();

    let header = format!("{:<20}\t{:<30}\t{}", "BACKEND", "NAME", "DESCRIPTION");

    if plain_text {
        println!("{}", header);
        for line in &lines {
            println!("{}", line);
        }
        return Ok(());
    }

    fzf::require_fzf()?;

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4,5"), &lines)?;

    Ok(())
}

pub fn run_all(plain_text: bool) -> anyhow::Result<()> {
    let results = db::list_all_topics()?;
    if results.is_empty() {
        println!("No man pages indexed. Try installing a backend first.");
        return Ok(());
    }

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name, description)| {
            let display_name = format!("{}({})", name, section);
            format!("{}\t{}\t{}\t{:<30}\t{}", backend, section, name, display_name, description)
        })
        .collect();

    let header = format!("{:<20}\t{:<30}\t{}", "BACKEND", "NAME", "DESCRIPTION");

    if plain_text {
        println!("{}", header);
        for line in &lines {
            println!("{}", line);
        }
        return Ok(());
    }

    fzf::require_fzf()?;

    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "uniman".to_string());

    let execute_template = format!("{} {{1}} {{2}} {{3}}", exe);

    fzf::browse(&header, &execute_template, Some("1,4,5"), &lines)?;

    Ok(())
}