use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
    email: String,
}

pub fn parse_user_json(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn extract_field(json_str: &str, field: &str) -> Result<String> {
    let v: Value = serde_json::from_str(json_str)?;
    
    match v.get(field) {
        Some(value) => {
            if value.is_string() {
                Ok(value.as_str().unwrap().to_string())
            } else {
                Ok(value.to_string())
            }
        }
        None => Err(serde_json::Error::custom(format!("Field '{}' not found", field))),
    }
}

pub fn validate_json_schema(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_user() {
        let json = r#"{"name":"Alice","age":30,"email":"alice@example.com"}"#;
        let result = parse_user_json(json);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_extract_existing_field() {
        let json = r#"{"name":"Bob","age":25}"#;
        let result = extract_field(json, "name");
        assert_eq!(result.unwrap(), "Bob");
    }

    #[test]
    fn test_validate_correct_json() {
        let json = r#"{"key":"value"}"#;
        assert!(validate_json_schema(json));
    }
}