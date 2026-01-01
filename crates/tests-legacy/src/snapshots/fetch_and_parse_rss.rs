use rss::Channel;
use std::error::Error;

pub fn fetch_and_parse_rss(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    let titles: Vec<String> = channel
        .items()
        .iter()
        .filter_map(|item| item.title().map(String::from))
        .collect();

    Ok(titles)
}