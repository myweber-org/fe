use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_and_parse_rss(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com/feed.rss";
    let channel = fetch_and_parse_rss(url).await?;
    println!("Feed Title: {}", channel.title());
    for item in channel.items() {
        if let Some(title) = item.title() {
            println!("Item: {}", title);
        }
    }
    Ok(())
}