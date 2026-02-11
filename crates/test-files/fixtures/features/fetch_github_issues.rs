
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
    let url = format!("https://api.github.com/repos/{}/{}/issues?state=open", owner, repo);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-Script")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Value = response.json().await?;
        println!("Open issues for {}/{}:", owner, repo);
        for issue in issues.as_array().unwrap_or(&vec![]) {
            if let Some(title) = issue["title"].as_str() {
                if let Some(number) = issue["number"].as_u64() {
                    println!("#{}: {}", number, title);
                }
            }
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}