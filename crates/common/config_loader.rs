
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let config_path = format!("config/{}.toml", env);
        Self::from_file(config_path)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }

        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }

        if self.database.port == 0 {
            errors.push("Database port cannot be 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}