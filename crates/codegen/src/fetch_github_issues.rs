use reqwest;
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
        .header("User-Agent", "rust-issue-fetcher")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Value> = response.json().await?;
        println!("Open issues for {}/{}:", owner, repo);
        for issue in issues {
            if let Some(state) = issue.get("state").and_then(|s| s.as_str()) {
                if state == "open" {
                    let number = issue["number"].as_i64().unwrap_or(0);
                    let title = issue["title"].as_str().unwrap_or("No title");
                    println!("#{}: {}", number, title);
                }
            }
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}