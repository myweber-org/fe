
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }
            
            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();
            
            value = Self::substitute_env_vars(&value);
            values.insert(key, value);
        }
        
        Ok(Config { values })
    }
    
    fn substitute_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        
        for (key, value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &value);
        }
        
        result
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}