use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
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

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
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
    fn test_env_var_priority() {
        env::set_var("TEST_KEY", "env_value");
        
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");
        
        assert_eq!(config.get("TEST_KEY"), Some("env_value".to_string()));
        
        env::remove_var("TEST_KEY");
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_pattern = Regex::new(r"\$\{([^}]+)\}")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                // Replace environment variable references
                let mut processed_value = value.to_string();
                for cap in var_pattern.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            host=localhost
            port=8080
            # This is a comment
            timeout=30
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "9090");
        env::set_var("DB_HOST", "postgres-server");

        let content = r#"
            server_port=${APP_PORT}
            database_host=${DB_HOST}
            static_value=constant
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("server_port"), Some(&"9090".to_string()));
        assert_eq!(config.get("database_host"), Some(&"postgres-server".to_string()));
        assert_eq!(config.get("static_value"), Some(&"constant".to_string()));
    }

    #[test]
    fn test_file_loading() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.conf");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "key1=value1\nkey2=value2").unwrap();

        let config = Config::from_file(file_path.to_str().unwrap()).unwrap();
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let content = "existing_key=actual_value";
        let config = Config::from_str(content).unwrap();

        assert_eq!(config.get_or_default("existing_key", "default"), "actual_value");
        assert_eq!(config.get_or_default("missing_key", "default_value"), "default_value");
    }
}