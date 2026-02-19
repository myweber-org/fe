use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    name: String,
    age: u8,
    email: String,
}

pub fn parse_json_string(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn create_json_from_user(user: &User) -> Result<String> {
    let json_string = serde_json::to_string(user)?;
    Ok(json_string)
}

pub fn extract_field(json_str: &str, field: &str) -> Result<Option<Value>> {
    let v: Value = serde_json::from_str(json_str)?;
    Ok(v.get(field).cloned())
}