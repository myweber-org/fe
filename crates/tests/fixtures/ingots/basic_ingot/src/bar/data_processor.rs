use std::collections::HashMap;

#[derive(Debug)]
pub struct UserData {
    id: u32,
    name: String,
    email: String,
    age: u8,
}

impl UserData {
    pub fn new(id: u32, name: String, email: String, age: u8) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        if age > 120 {
            return Err("Age must be less than 120".to_string());
        }
        
        Ok(Self { id, name, email, age })
    }
    
    pub fn to_json(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.to_string());
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("age".to_string(), self.age.to_string());
        map
    }
}

pub fn process_user_data(users: Vec<UserData>) -> Vec<HashMap<String, String>> {
    users
        .into_iter()
        .filter(|user| user.age >= 18)
        .map(|user| user.to_json())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_user_creation() {
        let user = UserData::new(1, "John".to_string(), "john@example.com".to_string(), 25);
        assert!(user.is_ok());
    }
    
    #[test]
    fn test_invalid_email() {
        let user = UserData::new(2, "Jane".to_string(), "invalid-email".to_string(), 30);
        assert!(user.is_err());
    }
    
    #[test]
    fn test_process_adult_users() {
        let users = vec![
            UserData::new(1, "Alice".to_string(), "alice@example.com".to_string(), 17).unwrap(),
            UserData::new(2, "Bob".to_string(), "bob@example.com".to_string(), 25).unwrap(),
        ];
        
        let processed = process_user_data(users);
        assert_eq!(processed.len(), 1);
    }
}