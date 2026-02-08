use reqwest;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://httpbin.org/get";
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    println!("Response body: {}", body);
    Ok(())
}