use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("Usage: {} <github_username>", args[0]);
        return Ok(());
    };

    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        println!("Repositories for user '{}':", username);
        for repo in repos {
            println!("- {} ({})", repo.name, repo.html_url);
            if let Some(desc) = repo.description {
                println!("  Description: {}", desc);
            }
            println!("  Stars: {}", repo.stargazers_count);
        }
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
    }

    Ok(())
}