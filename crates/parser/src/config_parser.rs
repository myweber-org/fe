use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config_map.insert(config_key, value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map.get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or("Missing database_url configuration")?;

        let port = map.get("port")
            .map(|s| s.parse::<u16>())
            .unwrap_or(Ok(8080))
            .map_err(|e| format!("Invalid port value: {}", e))?;

        let debug_mode = map.get("debug_mode")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let api_keys = map.get("api_keys")
            .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("Port must be greater than 0".to_string());
        }

        if self.api_keys.is_empty() {
            errors.push("At least one API key must be provided".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "database_url=postgres://localhost/test").unwrap();
        writeln!(temp_file, "port=3000").unwrap();
        writeln!(temp_file, "debug_mode=true").unwrap();
        writeln!(temp_file, "api_keys=key1,key2,key3").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            port: 0,
            debug_mode: false,
            api_keys: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}