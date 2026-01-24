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
                config.set(key.trim(), value.trim());
            }
        }

        Ok(config)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_with_env_fallback(&self, key: &str) -> Option<String> {
        if let Some(value) = self.get(key) {
            return Some(value.to_string());
        }

        env::var(key).ok()
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get_with_env_fallback(key)
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost"));
        assert_eq!(config.get("API_KEY"), Some("secret123"));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_fallback() {
        env::set_var("TEST_ENV_VAR", "env_value");
        let config = Config::new();
        assert_eq!(
            config.get_with_env_fallback("TEST_ENV_VAR"),
            Some("env_value".to_string())
        );
    }
}