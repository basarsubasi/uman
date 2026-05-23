use crate::db;

pub fn run_filename(topic: &str) -> anyhow::Result<()> {
    let results = db::search_by_name(topic)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names.", topic);
        return Ok(());
    }

    println!("{:<20} {:<10} {}", "BACKEND", "SECTION", "NAME");
    for (backend, section, name) in &results {
        println!("{:<20} {:<10} {}", backend, section, name);
    }
    Ok(())
}

pub fn run_keyword(keyword: &str) -> anyhow::Result<()> {
    let results = db::search_by_keyword(keyword)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names and descriptions.", keyword);
        return Ok(());
    }

    println!("{:<20} {:<10} {:<30} {}", "BACKEND", "SECTION", "NAME", "DESCRIPTION");
    for (backend, section, name, description) in &results {
        let desc_display = if description.len() > 50 {
            format!("{}...", &description[..47])
        } else {
            description.clone()
        };
        println!("{:<20} {:<10} {:<30} {}", backend, section, name, desc_display);
    }
    Ok(())
}