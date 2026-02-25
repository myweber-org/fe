use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
        let config_content = fs::read_to_string(config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_lowercase();
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}

pub fn load_config() -> AppConfig {
    match AppConfig::from_env() {
        Ok(config) => {
            println!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    }
}