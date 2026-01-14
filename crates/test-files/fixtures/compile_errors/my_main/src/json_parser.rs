use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

pub fn parse_user_json(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn create_user_json(user: &User) -> Result<String> {
    let json = serde_json::to_string(user)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parsing() {
        let json_data = r#"
        {
            "id": 42,
            "username": "rustacean",
            "email": "user@example.com",
            "active": true
        }
        "#;

        let result = parse_user_json(json_data);
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "rustacean");
        assert_eq!(user.email, "user@example.com");
        assert!(user.active);
    }

    #[test]
    fn test_json_creation() {
        let user = User {
            id: 100,
            username: String::from("testuser"),
            email: String::from("test@example.com"),
            active: false,
        };

        let result = create_user_json(&user);
        assert!(result.is_ok());
        
        let json_str = result.unwrap();
        assert!(json_str.contains("\"id\":100"));
        assert!(json_str.contains("\"username\":\"testuser\""));
    }
}