use std::fs;
use std::collections::HashMap;
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_tls: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

pub fn load_config(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_content)?;
    Ok(config)
}

pub fn validate_config(config: &AppConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if config.server.port == 0 {
        errors.push("Server port cannot be zero".to_string());
    }

    if config.database.max_connections == 0 {
        errors.push("Database max connections must be greater than zero".to_string());
    }

    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        errors.push(format!("Invalid log level: {}", config.logging.level));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn get_default_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_tls: false,
        },
        database: DatabaseConfig {
            url: "postgresql://localhost/mydb".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            file_path: Some("app.log".to_string()),
            enable_console: true,
        },
    }
}

pub fn merge_configs(base: &AppConfig, override_config: &HashMap<String, String>) -> AppConfig {
    let mut merged = base.clone();

    for (key, value) in override_config {
        match key.as_str() {
            "server.host" => merged.server.host = value.clone(),
            "server.port" => merged.server.port = value.parse().unwrap_or(base.server.port),
            "server.enable_tls" => merged.server.enable_tls = value.parse().unwrap_or(base.server.enable_tls),
            "database.url" => merged.database.url = value.clone(),
            "database.max_connections" => merged.database.max_connections = value.parse().unwrap_or(base.database.max_connections),
            "database.timeout_seconds" => merged.database.timeout_seconds = value.parse().unwrap_or(base.database.timeout_seconds),
            "logging.level" => merged.logging.level = value.clone(),
            "logging.file_path" => merged.logging.file_path = Some(value.clone()),
            "logging.enable_console" => merged.logging.enable_console = value.parse().unwrap_or(base.logging.enable_console),
            _ => {}
        }
    }

    merged
}

impl Clone for AppConfig {
    fn clone(&self) -> Self {
        AppConfig {
            server: ServerConfig {
                host: self.server.host.clone(),
                port: self.server.port,
                enable_tls: self.server.enable_tls,
            },
            database: DatabaseConfig {
                url: self.database.url.clone(),
                max_connections: self.database.max_connections,
                timeout_seconds: self.database.timeout_seconds,
            },
            logging: LoggingConfig {
                level: self.logging.level.clone(),
                file_path: self.logging.file_path.clone(),
                enable_console: self.logging.enable_console,
            },
        }
    }
}