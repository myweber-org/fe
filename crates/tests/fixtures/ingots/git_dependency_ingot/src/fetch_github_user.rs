
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    public_repos: u32,
    followers: u32,
    following: u32,
}

async fn fetch_github_user(username: &str) -> Result<GitHubUser, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;
    
    response.json::<GitHubUser>().await
}

#[tokio::main]
async fn main() {
    match fetch_github_user("octocat").await {
        Ok(user) => {
            println!("User: {}", user.login);
            println!("ID: {}", user.id);
            println!("Name: {:?}", user.name);
            println!("Public Repos: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
        }
        Err(e) => eprintln!("Error fetching user: {}", e),
    }
}use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    location: Option<String>,
    public_repos: u32,
    followers: u32,
    following: u32,
}

async fn fetch_github_user(username: &str) -> Result<GitHubUser, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "RustFetchApp")
        .send()
        .await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(format!("Failed to fetch user: HTTP {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }

    let username = &args[1];
    match fetch_github_user(username).await {
        Ok(user) => {
            println!("GitHub User Information:");
            println!("  Username: {}", user.login);
            println!("  ID: {}", user.id);
            println!("  Profile URL: {}", user.html_url);
            println!("  Avatar URL: {}", user.avatar_url);
            if let Some(name) = user.name {
                println!("  Name: {}", name);
            }
            if let Some(company) = user.company {
                println!("  Company: {}", company);
            }
            if let Some(location) = user.location {
                println!("  Location: {}", location);
            }
            println!("  Public Repositories: {}", user.public_repos);
            println!("  Followers: {}", user.followers);
            println!("  Following: {}", user.following);
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}