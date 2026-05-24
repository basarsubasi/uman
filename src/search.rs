use crate::db;
use crate::fzf;

const CELL_PAD: &str = " ";

fn pad_cell(value: &str) -> String {
    format!("{CELL_PAD}{value}{CELL_PAD}")
}

pub fn run_filename(topic: &str, plain_text: bool) -> anyhow::Result<()> {
    let results = db::search_by_name(topic)?;
    if results.is_empty() {
        println!("No matches for '{}' in page names.", topic);
        return Ok(());
    }

    let lines: Vec<String> = results
        .iter()
        .map(|(backend, section, name)| {
            let section_col = section.to_string();
            let display_name = format!("{}({})", name, section_col);
            let backend_col = format!("{backend:<20}");
            let display_col = format!("{display_name:<30}");
            format!(
                "{}\t{}\t{}\t{}",
                pad_cell(&backend_col),
                pad_cell(&section_col),
                pad_cell(name),
                pad_cell(&display_col)
            )
        })
        .collect();

    let header_backend = pad_cell(&format!("{:<20}", "BACKEND"));
    let header_name = pad_cell("NAME");
    let header = format!("{}\t{}", header_backend, header_name);

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
            let section_col = section.to_string();
            let display_name = format!("{}({})", name, section_col);
            let backend_col = format!("{backend:<20}");
            let display_col = format!("{display_name:<30}");
            format!(
                "{}\t{}\t{}\t{}\t{}",
                pad_cell(&backend_col),
                pad_cell(&section_col),
                pad_cell(name),
                pad_cell(&display_col),
                pad_cell(description)
            )
        })
        .collect();

    let header_backend = pad_cell(&format!("{:<20}", "BACKEND"));
    let header_name = pad_cell(&format!("{:<30}", "NAME"));
    let header_desc = pad_cell("DESCRIPTION");
    let header = format!("{}\t{}\t{}", header_backend, header_name, header_desc);

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
            let section_col = section.to_string();
            let display_name = format!("{}({})", name, section_col);
            let backend_col = format!("{backend:<20}");
            let display_col = format!("{display_name:<30}");
            format!(
                "{}\t{}\t{}\t{}\t{}",
                pad_cell(&backend_col),
                pad_cell(&section_col),
                pad_cell(name),
                pad_cell(&display_col),
                pad_cell(description)
            )
        })
        .collect();

    let header_backend = pad_cell(&format!("{:<20}", "BACKEND"));
    let header_name = pad_cell(&format!("{:<30}", "NAME"));
    let header_desc = pad_cell("DESCRIPTION");
    let header = format!("{}\t{}\t{}", header_backend, header_name, header_desc);

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