use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }
        
        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key) || env::var(key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_environment_variable_priority() {
        env::set_var("TEST_ENV_VAR", "env_value");
        
        let mut config = Config::new();
        config.set("TEST_ENV_VAR", "file_value");
        
        assert_eq!(config.get("TEST_ENV_VAR"), Some("env_value".to_string()));
    }

    #[test]
    fn test_get_with_default() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING", "default_value"), "default_value");
    }

    #[test]
    fn test_contains_key() {
        env::set_var("ENV_KEY", "value");
        
        let mut config = Config::new();
        config.set("FILE_KEY", "value");
        
        assert!(config.contains_key("ENV_KEY"));
        assert!(config.contains_key("FILE_KEY"));
        assert!(!config.contains_key("MISSING_KEY"));
    }
}