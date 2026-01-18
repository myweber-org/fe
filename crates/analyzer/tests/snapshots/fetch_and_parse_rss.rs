use rss::Channel;
use std::error::Error;

pub fn fetch_and_parse_rss(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    let items: Vec<String> = channel
        .items()
        .iter()
        .map(|item| {
            item.title()
                .unwrap_or("No title")
                .to_string()
        })
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_sample_rss() {
        let url = "https://example.com/feed.rss";
        let result = fetch_and_parse_rss(url);
        assert!(result.is_ok());
    }
}