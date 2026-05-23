use crate::db;

pub fn run(topic: &str) -> anyhow::Result<()> {
    let results = db::search(topic)?;
    if results.is_empty() {
        println!("No results found for '{topic}'.");
        return Ok(());
    }

    println!("{:<20} {:<10} {}", "BACKEND", "SECTION", "NAME");
    for (backend, section, name) in &results {
        println!("{:<20} {:<10} {}", backend, section, name);
    }
    Ok(())
}