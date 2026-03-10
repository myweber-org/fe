use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub api_port: u16,
    pub log_level: String,
    pub feature_flags: HashMap<String, bool>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVariable("DATABASE_URL".to_string()))?;
        
        let api_port = env::var("API_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue("API_PORT".to_string()))?;
        
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
        let mut feature_flags = HashMap::new();
        if let Ok(flags) = env::var("FEATURE_FLAGS") {
            for flag in flags.split(',') {
                let parts: Vec<&str> = flag.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let value = parts[1].trim().parse::<bool>().unwrap_or(false);
                    feature_flags.insert(key, value);
                }
            }
        }
        
        Ok(Self {
            database_url,
            api_port,
            log_level,
            feature_flags,
        })
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database_url.is_empty() {
            return Err(ConfigError::InvalidValue("DATABASE_URL".to_string()));
        }
        
        if self.api_port == 0 {
            return Err(ConfigError::InvalidValue("API_PORT".to_string()));
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::InvalidValue("LOG_LEVEL".to_string()));
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(String),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVariable(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidValue(var) => write!(f, "Invalid value for variable: {}", var),
        }
    }
}

impl std::error::Error for ConfigError {}