use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let mut config = HashMap::new();

        if let Ok(contents) = fs::read_to_string("config.toml") {
            for line in contents.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    config.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }

        let database_url = config
            .get("database_url")
            .ok_or("Missing database_url configuration")?
            .clone();

        let api_key = config
            .get("api_key")
            .ok_or("Missing api_key configuration")?
            .clone();

        let debug_mode = config
            .get("debug_mode")
            .map(|v| v == "true")
            .unwrap_or(false);

        let port = config
            .get("port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        if self.api_key.len() < 16 {
            return Err("API key must be at least 16 characters".to_string());
        }

        if self.port == 0 {
            return Err("Port cannot be zero".to_string());
        }

        Ok(())
    }
}