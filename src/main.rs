mod utils;

use std::env;
use quick_xml::events::Event;
use quick_xml::Reader;
use crate::utils::{construct_rss_feed_url, load_last_id, save_last_id};

struct Video {
    id: String,
    title: String,
}

fn fetch_feed(client: &reqwest::blocking::Client, url: &str) -> Result<String, reqwest::Error> {
    client.get(url).send()?.text()
}

enum CurrentField {
    VideoId,
    Title,
    None,
}

fn parse_feed(xml: &str, last_id: &str) -> Vec<Video> {
    let mut reader = Reader::from_str(xml);
    let mut videos: Vec<Video> = Vec::new();
    let mut current_field = CurrentField::None;
    let mut current_id = String::new();
    let mut current_title = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                current_field = match e.name().as_ref() {
                    b"yt:videoId" => CurrentField::VideoId,
                    b"title" => CurrentField::Title,
                    _ => CurrentField::None,
                };
            }
            Ok(Event::Text(e)) => {
                let text = reader.decoder().decode(e.as_ref()).unwrap_or_default().into_owned();
                match current_field {
                    CurrentField::VideoId => current_id = text,
                    CurrentField::Title => current_title = text,
                    CurrentField::None => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"entry" {
                    if current_id == last_id {
                        break;
                    }
                    videos.push(Video {
                        id: current_id.clone(),
                        title: current_title.clone(),
                    });
                    current_id.clear();
                    current_title.clear();
                }
                current_field = CurrentField::None;
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    videos
}

fn send_update(client: &reqwest::blocking::Client, webhook_url: &str, message: &str) {
    let body = serde_json::json!({ "content": message });
    client
        .post(webhook_url)
        .json(&body)
        .send()
        .expect("failed to send webhook");
}

fn main() {
    dotenvy::dotenv().ok();
    let client = reqwest::blocking::Client::new();
    let webhook_url = env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL not set");
    let yt_channel_id = env::var("YT_CHANNEL_ID").expect("YT_CHANNEL_ID not set");
    let yt_rss_feed_url = construct_rss_feed_url(&yt_channel_id);

    let last_id = load_last_id();
    let xml = fetch_feed(&client, &yt_rss_feed_url).expect("failed to fetch feed");
    let new_videos = parse_feed(&xml, &last_id);

    if new_videos.is_empty() {
        println!("no new videos");
        return;
    }

    for video in &new_videos {
        let message = format!("📺 new video: {} https://youtube.com/watch?v={}", video.title, video.id);
        send_update(&client, &webhook_url, &message);
    }

    save_last_id(&new_videos[0].id);
}
