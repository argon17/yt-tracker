use quick_xml::events::Event;
use quick_xml::Reader;

pub struct Video {
    pub id: String,
    pub title: String,
}

enum CurrentField {
    VideoId,
    Title,
    None,
}

const YT_RSS_BASE_URL: &str = "https://www.youtube.com/feeds/videos.xml?channel_id=";

pub fn construct_rss_feed_url(channel_id: &str) -> String {
    format!("{YT_RSS_BASE_URL}{channel_id}")
}

pub fn fetch_feed(client: &reqwest::blocking::Client, url: &str) -> anyhow::Result<String> {
    Ok(client.get(url).send()?.text()?)
}

pub fn parse_feed(xml: &str, last_id: &str) -> Vec<Video> {
    let mut reader = Reader::from_str(xml);
    let mut videos: Vec<Video> = Vec::new();
    let mut current_field = CurrentField::None;
    let mut current_id = String::new();
    let mut current_title = String::new();
    let mut inside_entry = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                current_field = match e.name().as_ref() {
                    b"entry" => {
                        inside_entry = true;
                        CurrentField::None
                    }
                    b"yt:videoId" if inside_entry => CurrentField::VideoId,
                    b"title" if inside_entry => CurrentField::Title,
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
                    inside_entry = false;
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
    videos.reverse();
    videos
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEWER_ID: &str = "ABCDEFGHIJK";
    const OLDER_ID: &str = "LMNOPQRSTUV";

    const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015">
  <title>My Channel</title>
  <entry>
    <yt:videoId>ABCDEFGHIJK</yt:videoId>
    <title>Newer Video</title>
  </entry>
  <entry>
    <yt:videoId>LMNOPQRSTUV</yt:videoId>
    <title>Older Video</title>
  </entry>
</feed>"#;

    #[test]
    fn returns_all_videos_in_chronological_order() {
        let videos = parse_feed(FEED, "");
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].id, OLDER_ID);
        assert_eq!(videos[1].id, NEWER_ID);
    }

    #[test]
    fn returns_only_videos_newer_than_last_id() {
        let videos = parse_feed(FEED, OLDER_ID);
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, NEWER_ID);
        assert_eq!(videos[0].title, "Newer Video");
    }

    #[test]
    fn returns_empty_when_already_up_to_date() {
        let videos = parse_feed(FEED, NEWER_ID);
        assert!(videos.is_empty());
    }

    #[test]
    fn channel_title_not_captured_as_video_title() {
        let videos = parse_feed(FEED, "");
        assert_eq!(videos[0].title, "Older Video");
        assert_eq!(videos[1].title, "Newer Video");
    }

    #[test]
    fn construct_rss_feed_url_formats_correctly() {
        let url = construct_rss_feed_url("UCABCDEFGHIJKLMNOPQRSTUV");
        assert_eq!(
            url,
            "https://www.youtube.com/feeds/videos.xml?channel_id=UCABCDEFGHIJKLMNOPQRSTUV"
        );
    }
}
