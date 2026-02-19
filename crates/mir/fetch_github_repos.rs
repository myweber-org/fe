use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
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
        let repos: Vec<Repository> = response.json().await?;
        println!("Repositories for user '{}':", username);
        for repo in repos {
            println!("- {} ({})", repo.name, repo.full_name);
            if let Some(desc) = repo.description {
                println!("  Description: {}", desc);
            }
            println!("  Stars: {}", repo.stargazers_count);
            println!("  URL: {}", repo.html_url);
            println!();
        }
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
    }

    Ok(())
}use reqwest;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        "rust-lang".to_string()
    };

    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Fetcher")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Value = response.json().await?;
        println!("Repositories for {}:", username);
        for repo in repos.as_array().unwrap_or(&vec![]) {
            let name = repo["name"].as_str().unwrap_or("N/A");
            let description = repo["description"].as_str().unwrap_or("No description");
            let stars = repo["stargazers_count"].as_u64().unwrap_or(0);
            let forks = repo["forks_count"].as_u64().unwrap_or(0);
            println!("  - {}: {}", name, description);
            println!("    Stars: {}, Forks: {}", stars, forks);
        }
    } else {
        eprintln!("Failed to fetch repositories for {}", username);
    }

    Ok(())
}