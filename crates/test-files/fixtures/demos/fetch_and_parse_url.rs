use reqwest;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.rust-lang.org";
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("title").unwrap();

    if let Some(title_element) = document.select(&selector).next() {
        let title = title_element.text().collect::<String>();
        println!("Page title: {}", title);
    } else {
        println!("No title found on the page.");
    }

    Ok(())
}