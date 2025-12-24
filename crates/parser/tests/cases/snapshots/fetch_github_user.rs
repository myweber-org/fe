
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub public_repos: u32,
}

pub async fn fetch_github_user(username: &str) -> Result<GitHubUser, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .header("User-Agent", "rust-api-client")
        .send()
        .await?;
    
    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(format!("Failed to fetch user: HTTP {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_fetch_github_user_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/testuser")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "login": "testuser",
                "id": 12345,
                "avatar_url": "https://example.com/avatar.jpg",
                "html_url": "https://github.com/testuser",
                "name": "Test User",
                "company": "Test Corp",
                "location": "Test City",
                "public_repos": 42
            }"#)
            .create_async()
            .await;

        let user = fetch_github_user("testuser").await;
        mock.assert_async().await;
        
        assert!(user.is_ok());
        let user_data = user.unwrap();
        assert_eq!(user_data.login, "testuser");
        assert_eq!(user_data.id, 12345);
        assert_eq!(user_data.public_repos, 42);
    }

    #[tokio::test]
    async fn test_fetch_github_user_not_found() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/users/nonexistent")
            .with_status(404)
            .create_async()
            .await;

        let result = fetch_github_user("nonexistent").await;
        mock.assert_async().await;
        
        assert!(result.is_err());
    }
}