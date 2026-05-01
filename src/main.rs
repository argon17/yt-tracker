mod discord;
mod feed;
mod state;

const YT_WATCH_BASE_URL: &str = "https://youtube.com/watch?v=";

use std::env;

struct Config {
    webhook_url: String,
    channel_id: String,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            webhook_url: env::var("DISCORD_WEBHOOK_URL")?,
            channel_id: env::var("YT_CHANNEL_ID")?,
        })
    }
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    let client = reqwest::blocking::Client::new();
    let rss_url = feed::construct_rss_feed_url(&config.channel_id);

    let last_id = state::load_last_id();
    let xml = feed::fetch_feed(&client, &rss_url)?;
    let new_videos = feed::parse_feed(&xml, &last_id);

    if new_videos.is_empty() {
        println!("no new videos");
        return Ok(());
    }

    // First run: no saved state yet — record the latest ID without spamming Discord.
    if last_id.is_empty() {
        let latest = new_videos.last().unwrap();
        state::save_last_id(&latest.id)?;
        println!("first run: saved latest id ({}), no notifications sent", latest.id);
        return Ok(());
    }

    for video in &new_videos {
        let message = format!(
            "📺 new video: {} {YT_WATCH_BASE_URL}{}",
            video.title, video.id
        );
        discord::send_update(&client, &config.webhook_url, &message)?;
    }

    state::save_last_id(&new_videos.last().unwrap().id)?;
    Ok(())
}
