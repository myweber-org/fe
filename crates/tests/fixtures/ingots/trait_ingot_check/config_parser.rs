
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
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map
            .get("DATABASE_URL")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/mydb".to_string());

        let port = map
            .get("PORT")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);

        let debug_mode = map
            .get("DEBUG")
            .map(|s| s.to_lowercase())
            .or_else(|| env::var("DEBUG").ok())
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let api_keys = map
            .get("API_KEYS")
            .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
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
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost:5432/test").unwrap();
        writeln!(temp_file, "PORT=3000").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEYS=key1,key2,key3").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/test");
        assert_eq!(config.port, 3000);
        assert!(config.debug_mode);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
    }

    #[test]
    fn test_config_defaults() {
        let map = HashMap::new();
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.port, 8080);
        assert!(!config.debug_mode);
        assert!(config.api_keys.is_empty());
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
        assert!(errors.contains(&"DATABASE_URL cannot be empty".to_string()));
        assert!(errors.contains(&"PORT must be greater than 0".to_string()));
    }
}