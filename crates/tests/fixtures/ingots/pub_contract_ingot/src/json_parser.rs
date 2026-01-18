use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
    preferences: HashMap<String, Value>,
}

#[derive(Debug)]
struct ParseError {
    details: String,
}

impl ParseError {
    fn new(msg: &str) -> Self {
        ParseError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError: {}", self.details)
    }
}

impl Error for ParseError {}

fn validate_json_structure(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str)
}

fn parse_user_data(json_str: &str) -> std::result::Result<User, Box<dyn Error>> {
    let value = validate_json_structure(json_str)?;
    
    if !value.is_object() {
        return Err(Box::new(ParseError::new("JSON must be an object")));
    }
    
    let user: User = serde_json::from_str(json_str)?;
    
    if user.name.is_empty() {
        return Err(Box::new(ParseError::new("User name cannot be empty")));
    }
    
    if !user.email.contains('@') {
        return Err(Box::new(ParseError::new("Invalid email format")));
    }
    
    Ok(user)
}

fn process_user_json(json_input: &str) {
    match parse_user_data(json_input) {
        Ok(user) => {
            println!("Successfully parsed user data:");
            println!("ID: {}", user.id);
            println!("Name: {}", user.name);
            println!("Email: {}", user.email);
            println!("Active: {}", user.active);
            println!("Preferences: {:?}", user.preferences);
        }
        Err(e) => {
            eprintln!("Failed to parse user data: {}", e);
        }
    }
}

fn main() {
    let valid_json = r#"
    {
        "id": 12345,
        "name": "John Doe",
        "email": "john@example.com",
        "active": true,
        "preferences": {
            "theme": "dark",
            "notifications": true,
            "language": "en"
        }
    }
    "#;
    
    let invalid_json = r#"
    {
        "id": 67890,
        "name": "",
        "email": "invalid-email",
        "active": false,
        "preferences": {}
    }
    "#;
    
    println!("Processing valid JSON:");
    process_user_json(valid_json);
    
    println!("\nProcessing invalid JSON:");
    process_user_json(invalid_json);
}