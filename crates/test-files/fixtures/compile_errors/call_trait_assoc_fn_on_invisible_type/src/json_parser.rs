use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: String,
    active: bool,
}

fn parse_json_file(file_path: &str) -> Result<Vec<User>> {
    let data = fs::read_to_string(file_path)?;
    let users: Vec<User> = serde_json::from_str(&data)?;
    Ok(users)
}

fn validate_user(user: &User) -> bool {
    !user.name.is_empty() && user.age > 0 && user.email.contains('@')
}

fn pretty_print_users(users: &[User]) {
    println!("Total users: {}", users.len());
    for (i, user) in users.iter().enumerate() {
        println!("\nUser #{}:", i + 1);
        println!("  Name: {}", user.name);
        println!("  Age: {}", user.age);
        println!("  Email: {}", user.email);
        println!("  Active: {}", user.active);
        println!("  Valid: {}", validate_user(user));
    }
}

fn filter_active_users(users: &[User]) -> Vec<&User> {
    users.iter().filter(|user| user.active).collect()
}

fn calculate_average_age(users: &[User]) -> f64 {
    if users.is_empty() {
        return 0.0;
    }
    let total_age: u32 = users.iter().map(|user| user.age).sum();
    total_age as f64 / users.len() as f64
}

fn main() -> Result<()> {
    let file_path = "users.json";
    
    match parse_json_file(file_path) {
        Ok(users) => {
            println!("Successfully parsed JSON file");
            pretty_print_users(&users);
            
            let active_users = filter_active_users(&users);
            println!("\nActive users: {}", active_users.len());
            
            let avg_age = calculate_average_age(&users);
            println!("Average age: {:.2}", avg_age);
            
            let valid_users: Vec<&User> = users.iter()
                .filter(|user| validate_user(user))
                .collect();
            println!("Valid users: {}", valid_users.len());
        }
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user() {
        let valid_user = User {
            name: "John Doe".to_string(),
            age: 30,
            email: "john@example.com".to_string(),
            active: true,
        };
        
        let invalid_user = User {
            name: "".to_string(),
            age: 0,
            email: "invalid-email".to_string(),
            active: false,
        };
        
        assert!(validate_user(&valid_user));
        assert!(!validate_user(&invalid_user));
    }

    #[test]
    fn test_calculate_average_age() {
        let users = vec![
            User { name: "Alice".to_string(), age: 25, email: "alice@example.com".to_string(), active: true },
            User { name: "Bob".to_string(), age: 35, email: "bob@example.com".to_string(), active: false },
            User { name: "Charlie".to_string(), age: 30, email: "charlie@example.com".to_string(), active: true },
        ];
        
        assert_eq!(calculate_average_age(&users), 30.0);
        assert_eq!(calculate_average_age(&[]), 0.0);
    }
}