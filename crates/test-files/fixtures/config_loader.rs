use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::substitute_env_vars(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn substitute_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                match env::var(&var_name) {
                    Ok(val) => result.push_str(&val),
                    Err(_) => result.push_str(&format!("${{{}}}", var_name)),
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST").unwrap(), "localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("MISSING").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_ENV", "production");
        env::set_var("DB_HOST", "db.local");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${{APP_ENV}}").unwrap();
        writeln!(file, "DATABASE_HOST=${{DB_HOST}}").unwrap();
        writeln!(file, "UNSET_VAR=${{NONEXISTENT}}").unwrap();
        writeln!(file, "MIXED=prefix_${{APP_ENV}}_suffix").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("ENVIRONMENT").unwrap(), "production");
        assert_eq!(config.get("DATABASE_HOST").unwrap(), "db.local");
        assert_eq!(config.get("UNSET_VAR").unwrap(), "${NONEXISTENT}");
        assert_eq!(config.get("MIXED").unwrap(), "prefix_production_suffix");
    }

    #[test]
    fn test_invalid_format() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY_WITHOUT_VALUE").unwrap();

        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}use serde::Deserialize;
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
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }

        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            config.cache_ttl = cache_ttl.parse()?;
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }

        Ok(())
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 20,
                min_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    validate_config(&config)?;
    Ok(config)
}

pub fn load_config_or_default<P: AsRef<Path>>(path: P) -> AppConfig {
    match load_config(path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}, using defaults", e);
            AppConfig::default()
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections must be greater than or equal to min connections".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
            [server]
            host = "0.0.0.0"
            port = 9000
            timeout_seconds = 60

            [database]
            url = "postgresql://prod:5432/appdb"
            max_connections = 50
            min_connections = 10

            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            max_file_size_mb = 500
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_str).unwrap();
        
        let result = load_config(temp_file.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.logging.level, "debug");
    }
}