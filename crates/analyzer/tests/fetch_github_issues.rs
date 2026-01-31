use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let owner = "rust-lang";
    let repo = "rust";
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-Script")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        println!("Open issues for {}/{}:", owner, repo);
        for issue in issues.iter().filter(|i| i.state == "open").take(5) {
            println!("#{}: {}", issue.number, issue.title);
            println!("   URL: {}", issue.html_url);
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}