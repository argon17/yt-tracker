use std::fs;

const LAST_CHECKED_FILE: &str = ".last-checked";

pub fn load_last_id() -> String {
    fs::read_to_string(LAST_CHECKED_FILE)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn save_last_id(id: &str) -> anyhow::Result<()> {
    fs::write(LAST_CHECKED_FILE, id)?;
    Ok(())
}
