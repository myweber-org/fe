use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let repo_owner = "rust-lang";
    let repo_name = "rust";
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues?state=open",
        repo_owner, repo_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-Script")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        println!("Open issues for {}/{}:", repo_owner, repo_name);
        for issue in issues.iter().take(5) {
            println!("#{}: {} ({})", issue.number, issue.title, issue.state);
            println!("URL: {}", issue.html_url);
            println!("---");
        }
        println!("Total open issues: {}", issues.len());
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}