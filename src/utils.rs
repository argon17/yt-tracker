use std::fs;

const LAST_CHECKED_FILE: &str = ".last-checked";

pub fn construct_rss_feed_url(channel_id: &str) -> String {
    format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}")
}

pub fn load_last_id() -> String {
    fs::read_to_string(LAST_CHECKED_FILE)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn save_last_id(id: &str) {
    fs::write(LAST_CHECKED_FILE, id).expect("failed to save last checked id");
}
