
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub pool_size: u32,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeatureFlags {
    pub enable_caching: bool,
    pub enable_logging: bool,
    pub maintenance_mode: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    EnvVarError(String),
}

impl AppConfig {
    pub fn load(config_path: &str) -> Result<Self, ConfigError> {
        let path = Path::new(config_path);
        if !path.exists() {
            return Err(ConfigError::FileNotFound(config_path.to_string()));
        }

        let config_content = fs::read_to_string(path)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let mut config: AppConfig = serde_json::from_str(&config_content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.apply_environment_overrides()?;
        Ok(config)
    }

    fn apply_environment_overrides(&mut self) -> Result<(), ConfigError> {
        let env_vars: HashMap<String, String> = env::vars().collect();

        if let Some(host) = env_vars.get("APP_SERVER_HOST") {
            self.server.host = host.clone();
        }

        if let Some(port_str) = env_vars.get("APP_SERVER_PORT") {
            self.server.port = port_str.parse()
                .map_err(|e| ConfigError::EnvVarError(format!("Invalid port: {}", e)))?;
        }

        if let Some(conn_str) = env_vars.get("APP_DB_CONNECTION") {
            self.database.connection_string = conn_str.clone();
        }

        if let Some(cache_flag) = env_vars.get("APP_ENABLE_CACHE") {
            self.features.enable_caching = cache_flag.to_lowercase() == "true";
        }

        if let Some(maintenance_flag) = env_vars.get("APP_MAINTENANCE_MODE") {
            self.features.maintenance_mode = maintenance_flag.to_lowercase() == "true";
        }

        Ok(())
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }

        if self.server.timeout_seconds > 300 {
            errors.push("Timeout cannot exceed 300 seconds".to_string());
        }

        if self.database.pool_size == 0 {
            errors.push("Database pool size must be greater than 0".to_string());
        }

        if self.database.max_connections < self.database.pool_size {
            errors.push("Max connections must be greater than or equal to pool size".to_string());
        }

        errors
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse configuration: {}", msg),
            ConfigError::EnvVarError(msg) => write!(f, "Environment variable error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}