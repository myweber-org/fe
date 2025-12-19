use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        config.parse_content(&content)?;
        Ok(config)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), String> {
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
            let value = parts[1].trim().to_string();

            if !self.validate_setting(&key, &value) {
                return Err(format!("Invalid value for {}: {}", key, value));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    fn validate_setting(&self, key: &str, value: &str) -> bool {
        match key {
            "timeout" => value.parse::<u32>().is_ok(),
            "retries" => value.parse::<u8>().is_ok(),
            "log_level" => ["debug", "info", "warn", "error"].contains(&value),
            _ => true,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_or_default(&self, key: &str) -> String {
        self.get(key)
            .map(|v| v.clone())
            .unwrap_or_else(|| "".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timeout=60\nretries=5\nlog_level=debug").unwrap();

        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"5".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timeout=invalid").unwrap();

        let result = Config::load_from_file(file.path());
        assert!(result.is_err());
    }
}