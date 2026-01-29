
use reqwest;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub public_repos: u32,
}

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("Network request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("User not found")]
    NotFound,
    #[error("API rate limit exceeded")]
    RateLimited,
    #[error("Unexpected status code: {0}")]
    UnexpectedStatus(reqwest::StatusCode),
}

pub async fn fetch_github_user(username: &str) -> Result<GitHubUser, GitHubError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/users/{}", username);
    
    let response = client
        .get(&url)
        .header("User-Agent", "rust-github-client")
        .send()
        .await?;
    
    match response.status() {
        reqwest::StatusCode::OK => {
            let user = response.json::<GitHubUser>().await?;
            Ok(user)
        }
        reqwest::StatusCode::NOT_FOUND => Err(GitHubError::NotFound),
        reqwest::StatusCode::FORBIDDEN => Err(GitHubError::RateLimited),
        status => Err(GitHubError::UnexpectedStatus(status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_user_success() {
        let mock = mock("GET", "/users/octocat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "login": "octocat",
                "id": 583231,
                "avatar_url": "https://avatars.githubusercontent.com/u/583231?v=4",
                "html_url": "https://github.com/octocat",
                "name": "The Octocat",
                "company": "GitHub",
                "public_repos": 8
            }"#)
            .create();

        let _guard = mockito::server_url();
        let result = fetch_github_user("octocat").await;
        
        mock.assert();
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.login, "octocat");
        assert_eq!(user.id, 583231);
        assert_eq!(user.name.unwrap(), "The Octocat");
    }

    #[tokio::test]
    async fn test_fetch_user_not_found() {
        let mock = mock("GET", "/users/nonexistent")
            .with_status(404)
            .create();

        let _guard = mockito::server_url();
        let result = fetch_github_user("nonexistent").await;
        
        mock.assert();
        assert!(matches!(result, Err(GitHubError::NotFound)));
    }
}