use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonParseError {
    #[error("Invalid JSON format: {0}")]
    InvalidFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Type mismatch for field {0}: expected {1}, got {2}")]
    TypeMismatch(String, String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub preferences: Option<Value>,
}

pub fn parse_user_json(json_str: &str) -> std::result::Result<UserData, JsonParseError> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| JsonParseError::InvalidFormat(e.to_string()))?;

    let id = value["id"]
        .as_u64()
        .ok_or_else(|| JsonParseError::TypeMismatch(
            "id".to_string(),
            "u64".to_string(),
            value["id"].to_string()
        ))?;

    let username = value["username"]
        .as_str()
        .ok_or_else(|| JsonParseError::TypeMismatch(
            "username".to_string(),
            "string".to_string(),
            value["username"].to_string()
        ))?
        .to_string();

    let email = value["email"]
        .as_str()
        .ok_or_else(|| JsonParseError::TypeMismatch(
            "email".to_string(),
            "string".to_string(),
            value["email"].to_string()
        ))?
        .to_string();

    let active = value["active"]
        .as_bool()
        .ok_or_else(|| JsonParseError::TypeMismatch(
            "active".to_string(),
            "bool".to_string(),
            value["active"].to_string()
        ))?;

    let preferences = value.get("preferences").cloned();

    Ok(UserData {
        id,
        username,
        email,
        active,
        preferences,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_parsing() {
        let json_data = r#"
        {
            "id": 42,
            "username": "rustacean",
            "email": "user@example.com",
            "active": true,
            "preferences": {"theme": "dark"}
        }
        "#;

        let result = parse_user_json(json_data);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "rustacean");
        assert_eq!(user.email, "user@example.com");
        assert!(user.active);
        assert!(user.preferences.is_some());
    }

    #[test]
    fn test_invalid_json_format() {
        let invalid_json = r#"{ invalid json }"#;
        let result = parse_user_json(invalid_json);
        assert!(result.is_err());
        match result.unwrap_err() {
            JsonParseError::InvalidFormat(_) => (),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_missing_required_field() {
        let json_missing_email = r#"
        {
            "id": 42,
            "username": "testuser",
            "active": true
        }
        "#;
        
        let result = parse_user_json(json_missing_email);
        assert!(result.is_err());
    }
}