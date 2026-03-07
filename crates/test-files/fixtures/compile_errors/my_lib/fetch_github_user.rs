
use reqwest;
use serde::Deserialize;
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }
    let username = &args[1];

    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::builder()
        .user_agent("github-user-fetcher/1.0")
        .build()?;

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        println!("GitHub User: {}", user.login);
        println!("ID: {}", user.id);
        println!("Profile: {}", user.html_url);
        if let Some(name) = user.name {
            println!("Name: {}", name);
        }
        if let Some(location) = user.location {
            println!("Location: {}", location);
        }
        println!("Public Repositories: {}", user.public_repos);
        println!("Followers: {}", user.followers);
        println!("Following: {}", user.following);
    } else {
        eprintln!("Error: Failed to fetch user '{}' (Status: {})", username, response.status());
        std::process::exit(1);
    }

    Ok(())
}