use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let max_connections = Self::get_value(map, "MAX_CONNECTIONS")?.parse()?;
        let debug_mode = Self::get_value(map, "DEBUG_MODE")?.parse()?;

        let mut api_keys = HashMap::new();
        for (key, value) in map {
            if key.starts_with("API_KEY_") {
                api_keys.insert(key[8..].to_string(), value.clone());
            }
        }

        Ok(Config {
            database_url,
            max_connections,
            debug_mode,
            api_keys,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
        map.get(key)
            .map(|s| s.clone())
            .or_else(|| env::var(key).ok())
            .ok_or_else(|| format!("Missing configuration: {}", key))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.max_connections == 0 {
            errors.push("MAX_CONNECTIONS must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}