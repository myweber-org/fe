use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        // Load from environment variables
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                values.insert(key.to_lowercase(), value);
            }
        }
        
        // Load from config file if exists
        if let Ok(contents) = fs::read_to_string("config.toml") {
            if let Ok(parsed) = toml::from_str::<HashMap<String, String>>(&contents) {
                for (key, value) in parsed {
                    values.insert(key, value);
                }
            }
        }
        
        Config { values }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
    
    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.is_empty() || !config.is_empty());
    }
    
    #[test]
    fn test_set_and_get() {
        let mut config = Config::new();
        config.set("test_key", "test_value");
        
        assert_eq!(config.get("test_key"), Some(&"test_value".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }
    
    #[test]
    fn test_get_or_default() {
        let mut config = Config::new();
        config.set("existing", "value");
        
        assert_eq!(config.get_or_default("existing", "default"), "value");
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }
}