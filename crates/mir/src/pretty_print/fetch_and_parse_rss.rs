use rss::Channel;
use std::error::Error;

pub fn fetch_and_parse_rss(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn print_feed_titles(channel: &Channel) {
    println!("Feed: {}", channel.title());
    for item in channel.items() {
        if let Some(title) = item.title() {
            println!("  - {}", title);
        }
    }
}