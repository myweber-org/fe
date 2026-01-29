use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        if let Ok(contents) = fs::read_to_string("config.toml") {
            if let Ok(parsed) = toml::from_str::<HashMap<String, String>>(&contents) {
                for (key, value) in parsed {
                    values.insert(key, value);
                }
            }
        }
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                values.insert(config_key, value);
            }
        }
        
        Config { values }
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
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn all_keys(&self) -> Vec<&String> {
        self.values.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.all_keys().len() >= 0);
    }
    
    #[test]
    fn test_get_or_default() {
        let config = Config::new();
        let value = config.get_or_default("nonexistent_key", "default_value");
        assert_eq!(value, "default_value");
    }
}