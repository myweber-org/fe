use std::error::Error;
use reqwest;
use rss::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rss_feed_url>", args[0]);
        std::process::exit(1);
    }

    let url = &args[1];
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("\nLatest Items:");
    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
        }
    }

    Ok(())
}