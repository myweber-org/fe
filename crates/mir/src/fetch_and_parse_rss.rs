use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn print_feed_items(channel: &Channel) {
    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("Feed Description: {}", channel.description());
    println!("\nLatest Items:");

    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
        }
        if let Some(link) = item.link() {
            println!("  Link: {}", link);
        }
        if let Some(desc) = item.description() {
            let preview: String = desc.chars().take(100).collect();
            println!("  Description: {}...", preview);
        }
        println!();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let feed_url = "https://example.com/feed.rss";
    match fetch_rss_feed(feed_url).await {
        Ok(channel) => {
            print_feed_items(&channel);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to fetch RSS feed: {}", e);
            Err(e)
        }
    }
}