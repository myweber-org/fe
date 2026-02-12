use reqwest;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Rss {
    channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    title: String,
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    link: String,
    pub_date: String,
}

pub async fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let rss: Rss = from_str(&response)?;
    Ok(rss.channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_rss_feed() {
        let url = "https://example.com/feed.rss";
        let result = fetch_rss_feed(url).await;
        assert!(result.is_err());
    }
}