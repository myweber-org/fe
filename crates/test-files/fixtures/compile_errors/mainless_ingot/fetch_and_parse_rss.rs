use reqwest;
use rss::Channel;
use std::error::Error;

pub fn fetch_and_parse_rss(url: &str) -> Result<Channel, Box<dyn Error>> {
    let body = reqwest::blocking::get(url)?.text()?;
    let channel = Channel::read_from(body.as_bytes())?;
    Ok(channel)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rss_feed_url>", args[0]);
        std::process::exit(1);
    }
    let url = &args[1];
    let channel = fetch_and_parse_rss(url)?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("\nLatest Items:");
    for item in channel.items().iter().take(5) {
        println!("- {}", item.title().unwrap_or("No title"));
    }
    Ok(())
}