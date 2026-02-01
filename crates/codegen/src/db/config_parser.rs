
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

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_with_env(&self, key: &str) -> Option<String> {
        if let Ok(env_value) = env::var(key) {
            return Some(env_value);
        }
        self.values.get(key).cloned()
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get_with_env(key).unwrap_or_else(|| default.to_string())
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
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "HOST=localhost\nPORT=8080\n# This is a comment\nTIMEOUT=30"
        )
        .unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        let mut config = Config::new();
        config.set("DATABASE_URL", "postgres://default/db");

        let value = config.get_with_env("DATABASE_URL");
        assert_eq!(value, Some("postgres://localhost/test".to_string()));
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        let value = config.get_or_default("LOG_LEVEL", "info");
        assert_eq!(value, "info");
    }
}