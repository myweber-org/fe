use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct RepoStats {
    name: String,
    stargazers_count: u32,
    forks_count: u32,
    open_issues_count: u32,
    language: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <owner> <repo>", args[0]);
        std::process::exit(1);
    }

    let owner = &args[1];
    let repo = &args[2];
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Stats-Fetcher")
        .send()
        .await?;

    if response.status().is_success() {
        let repo_stats: RepoStats = response.json().await?;
        println!("Repository: {}", repo_stats.name);
        println!("Stars: {}", repo_stats.stargazers_count);
        println!("Forks: {}", repo_stats.forks_count);
        println!("Open Issues: {}", repo_stats.open_issues_count);
        println!("Primary Language: {:?}", repo_stats.language);
    } else {
        eprintln!("Failed to fetch repository data. Status: {}", response.status());
    }

    Ok(())
}