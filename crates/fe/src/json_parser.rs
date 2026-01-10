use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
    preferences: HashMap<String, Value>,
}

fn parse_json_file(file_path: &str) -> Result<Vec<User>> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| serde_json::Error::io(e))?;
    
    let users: Vec<User> = serde_json::from_str(&content)?;
    
    validate_users(&users)?;
    
    Ok(users)
}

fn validate_users(users: &[User]) -> Result<()> {
    for user in users {
        if user.name.is_empty() {
            return Err(serde_json::Error::custom("User name cannot be empty"));
        }
        
        if !user.email.contains('@') {
            return Err(serde_json::Error::custom("Invalid email format"));
        }
        
        if user.id == 0 {
            return Err(serde_json::Error::custom("User ID must be positive"));
        }
    }
    
    Ok(())
}

fn process_users(users: &[User]) -> HashMap<String, Vec<&User>> {
    let mut grouped = HashMap::new();
    
    for user in users {
        let domain = user.email.split('@').nth(1).unwrap_or("unknown");
        grouped.entry(domain.to_string())
            .or_insert_with(Vec::new)
            .push(user);
    }
    
    grouped
}

fn generate_summary(users: &[User]) -> Value {
    let active_count = users.iter().filter(|u| u.active).count();
    let total_preferences: usize = users.iter()
        .map(|u| u.preferences.len())
        .sum();
    
    serde_json::json!({
        "total_users": users.len(),
        "active_users": active_count,
        "inactive_users": users.len() - active_count,
        "average_preferences": total_preferences as f64 / users.len() as f64,
        "unique_domains": process_users(users).len()
    })
}

fn save_processed_data(users: &[User], output_path: &str) -> Result<()> {
    let summary = generate_summary(users);
    let grouped = process_users(users);
    
    let output = serde_json::json!({
        "summary": summary,
        "users_by_domain": grouped,
        "raw_data": users
    });
    
    let json_string = serde_json::to_string_pretty(&output)?;
    fs::write(output_path, json_string)
        .map_err(|e| serde_json::Error::io(e))?;
    
    Ok(())
}

pub fn process_json_file(input_path: &str, output_path: &str) -> Result<()> {
    let users = parse_json_file(input_path)?;
    
    println!("Successfully parsed {} users", users.len());
    println!("Summary: {}", generate_summary(&users));
    
    save_processed_data(&users, output_path)?;
    
    println!("Processed data saved to {}", output_path);
    
    Ok(())
}