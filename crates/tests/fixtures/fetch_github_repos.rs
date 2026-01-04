use reqwest;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }
    let username = &args[1];
    let url = format!("https://api.github.com/users/{}/repos", username);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Value = response.json().await?;
        if let Some(repos_array) = repos.as_array() {
            println!("Repositories for {}:", username);
            for repo in repos_array {
                if let Some(name) = repo.get("name").and_then(|n| n.as_str()) {
                    if let Some(description) = repo.get("description").and_then(|d| d.as_str()) {
                        println!("- {}: {}", name, description.unwrap_or("No description"));
                    } else {
                        println!("- {}: No description", name);
                    }
                }
            }
        } else {
            println!("No repositories found or invalid response.");
        }
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
    }

    Ok(())
}