use reqwest;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::error::Error;

pub async fn fetch_and_parse_rss(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let mut reader = Reader::from_str(&response);
    reader.trim_text(true);

    let mut items = Vec::new();
    let mut buf = Vec::new();
    let mut current_title = String::new();
    let mut in_title = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"title" => {
                in_title = true;
                current_title.clear();
            }
            Ok(Event::Text(e)) if in_title => {
                current_title.push_str(&e.unescape()?);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"title" => {
                if !current_title.is_empty() {
                    items.push(current_title.clone());
                }
                in_title = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => (),
        }
        buf.clear();
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_rss() {
        let result = fetch_and_parse_rss("https://example.com/feed.rss").await;
        assert!(result.is_ok());
    }
}