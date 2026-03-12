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
}use reqwest;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <owner> <repo>", args[0]);
        std::process::exit(1);
    }

    let owner = &args[1];
    let repo = &args[2];
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-fetch-issues")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Value = response.json().await?;
        if let Some(issues_array) = issues.as_array() {
            for issue in issues_array {
                let number = issue["number"].as_i64().unwrap_or(0);
                let title = issue["title"].as_str().unwrap_or("No title");
                let state = issue["state"].as_str().unwrap_or("unknown");
                println!("#{} [{}] {}", number, state, title);
            }
        } else {
            println!("No issues found or invalid response format.");
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}