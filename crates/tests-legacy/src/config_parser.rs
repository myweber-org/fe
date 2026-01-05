
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                settings.insert(key, value);
            }
        }

        Self::from_map(&settings)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut settings = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                settings.insert(config_key, value);
            }
        }

        Self::from_map(&settings)
    }

    fn from_map(settings: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = settings
            .get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/mydb".to_string());

        let port = settings
            .get("port")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);

        let log_level = settings
            .get("log_level")
            .map(|s| s.to_string())
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());

        let cache_size = settings
            .get("cache_size")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("CACHE_SIZE").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(1000);

        Ok(Config {
            database_url,
            port,
            log_level,
            cache_size,
        })
    }

    pub fn merge(self, other: Self) -> Self {
        Config {
            database_url: other.database_url,
            port: other.port,
            log_level: other.log_level,
            cache_size: other.cache_size,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: "postgres://localhost:5432/mydb".to_string(),
            port: 8080,
            log_level: "info".to_string(),
            cache_size: 1000,
        }
    }
}