use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn print_feed_info(channel: &Channel) {
    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("Feed Description: {}", channel.description());
    println!("\nLatest Items:");

    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com/feed.rss";
    match fetch_rss_feed(url).await {
        Ok(channel) => {
            print_feed_info(&channel);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to fetch RSS feed: {}", e);
            Err(e)
        }
    }
}