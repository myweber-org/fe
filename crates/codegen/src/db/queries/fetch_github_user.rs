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
        .header("User-Agent", "Rust-GitHub-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

#[tokio::main]
async fn main() {
    match fetch_github_user("torvalds").await {
        Ok(user) => {
            println!("User: {}", user.login);
            println!("Name: {:?}", user.name);
            println!("Location: {:?}", user.location);
            println!("Public Repos: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
        }
        Err(e) => eprintln!("Error fetching user: {}", e),
    }
}