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
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let raw_value = parts[1].trim().to_string();
                let value = Self::interpolate_env_vars(&raw_value);
                values.insert(key, value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(input: &str) -> String {
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
        writeln!(file, "  TIMEOUT = 30  ").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_ENV", "production");
        env::set_var("DB_HOST", "db.local");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${{APP_ENV}}").unwrap();
        writeln!(file, "DATABASE_HOST=${{DB_HOST}}").unwrap();
        writeln!(file, "UNSET_VAR=${{NONEXISTENT}}").unwrap();
        writeln!(file, "MIXED=prefix_${{APP_ENV}}_suffix").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        assert_eq!(config.get("DATABASE_HOST"), Some(&"db.local".to_string()));
        assert_eq!(config.get("UNSET_VAR"), Some(&"${NONEXISTENT}".to_string()));
        assert_eq!(config.get("MIXED"), Some(&"prefix_production_suffix".to_string()));
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            enable_tls: false,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, ConfigError> {
    let path_ref = path.as_ref();
    
    if !path_ref.exists() {
        return Err(ConfigError::FileNotFound(
            path_ref.to_string_lossy().to_string()
        ));
    }

    let content = fs::read_to_string(path_ref)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;

    let mut config: ServerConfig = serde_yaml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;

    validate_config(&mut config)?;
    
    Ok(config)
}

fn validate_config(config: &mut ServerConfig) -> Result<(), ConfigError> {
    if config.port == 0 {
        return Err(ConfigError::ValidationError(
            "Port cannot be 0".to_string()
        ));
    }

    if config.max_connections == 0 {
        config.max_connections = ServerConfig::default().max_connections;
    }

    if config.host.trim().is_empty() {
        config.host = ServerConfig::default().host;
    }

    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &ServerConfig, path: P) -> Result<(), ConfigError> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;

    fs::write(path, yaml)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_save_and_load_config() {
        let mut config = ServerConfig::default();
        config.port = 9000;
        config.enable_tls = true;

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        save_config(&config, path).unwrap();
        let loaded_config = load_config(path).unwrap();

        assert_eq!(loaded_config.port, 9000);
        assert!(loaded_config.enable_tls);
    }

    #[test]
    fn test_validation() {
        let mut config = ServerConfig::default();
        config.port = 0;
        
        let result = validate_config(&mut config);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}