use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config.settings.insert(
                    key.trim().to_string(),
                    value.trim().trim_matches('"').to_string(),
                );
            }
        }

        Ok(config)
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        for key in required_keys {
            if !self.settings.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "HOST=localhost").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "TIMEOUT=\"30\"").unwrap();

        let config = Config::load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.settings.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.settings.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.settings.get("TIMEOUT"), Some(&"30".to_string()));
    }

    #[test]
    fn test_get_with_default() {
        let mut config = Config::new();
        config.settings.insert("HOST".to_string(), "127.0.0.1".to_string());

        assert_eq!(config.get_with_default("HOST", "localhost"), "127.0.0.1");
        assert_eq!(config.get_with_default("MISSING", "default_value"), "default_value");
    }

    #[test]
    fn test_validate_required() {
        let mut config = Config::new();
        config.settings.insert("API_KEY".to_string(), "secret".to_string());
        config.settings.insert("ENDPOINT".to_string(), "https://api.example.com".to_string());

        let result = config.validate_required(&["API_KEY", "ENDPOINT"]);
        assert!(result.is_ok());

        let result = config.validate_required(&["API_KEY", "MISSING"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["MISSING".to_string()]);
    }
}