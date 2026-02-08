use std::error::Error;
use reqwest;
use rss::Channel;

pub async fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rss_feed_url>", args[0]);
        std::process::exit(1);
    }
    let url = &args[1];
    let channel = fetch_rss_feed(url).await?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("Feed Description: {}", channel.description());
    println!("\nLatest Items:");
    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
        }
    }
    Ok(())
}