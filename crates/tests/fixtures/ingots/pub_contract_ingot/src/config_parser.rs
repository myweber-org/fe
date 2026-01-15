use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                value = self.substitute_env_vars(&value)?;
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }
        
        Ok(())
    }
    
    fn substitute_env_vars(&self, input: &str) -> Result<String, String> {
        let re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = input.to_string();
        
        for caps in re.captures_iter(input) {
            let var_name = &caps[1];
            match env::var(var_name) {
                Ok(val) => {
                    result = result.replace(&caps[0], &val);
                }
                Err(_) => {
                    return Err(format!("Environment variable '{}' not found", var_name));
                }
            }
        }
        
        Ok(result)
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            server_host = localhost
            server_port = 8080
            debug_mode = true
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_HOME", "/opt/myapp");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            data_dir = ${APP_HOME}/data
            log_dir = ${APP_HOME}/logs
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("data_dir"), Some(&"/opt/myapp/data".to_string()));
        assert_eq!(parser.get("log_dir"), Some(&"/opt/myapp/logs".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432"),
            max_connections: 10,
            timeout_seconds: 30,
            features: vec![String::from("logging"), String::from("caching")],
            metadata: HashMap::new(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::default();
        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config format at line {}", line_num + 1));
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match (current_section.as_str(), key) {
                ("database", "url") => config.database_url = value.to_string(),
                ("database", "max_connections") => {
                    config.max_connections = value.parse()
                        .map_err(|_| format!("Invalid number at line {}", line_num + 1))?
                }
                ("network", "timeout") => {
                    config.timeout_seconds = value.parse()
                        .map_err(|_| format!("Invalid number at line {}", line_num + 1))?
                }
                ("features", _) => {
                    config.features = value.split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                ("metadata", _) => {
                    config.metadata.insert(key.to_string(), value.to_string());
                }
                _ => return Err(format!("Unknown config key '{}' at line {}", key, line_num + 1)),
            }
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }
        
        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than zero"));
        }
        
        if self.timeout_seconds > 300 {
            return Err(String::from("Timeout cannot exceed 300 seconds"));
        }
        
        Ok(())
    }

    pub fn get_feature_status(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database_url, "postgresql://localhost:5432");
        assert_eq!(config.max_connections, 10);
        assert!(config.get_feature_status("logging"));
    }

    #[test]
    fn test_valid_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = postgresql://prod:5432").unwrap();
        writeln!(file, "max_connections = 20").unwrap();
        writeln!(file, "[network]").unwrap();
        writeln!(file, "timeout = 60").unwrap();
        writeln!(file, "[features]").unwrap();
        writeln!(file, "enabled = caching,monitoring").unwrap();
        writeln!(file, "[metadata]").unwrap();
        writeln!(file, "version = 2.1.0").unwrap();

        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://prod:5432");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.get_feature_status("caching"));
        assert_eq!(config.get_metadata("version"), Some(&"2.1.0".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = ").unwrap();

        let result = Config::from_file(file.path());
        assert!(result.is_err());
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
    pub file_path: String,
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
                file_path: "./logs/app.log".to_string(),
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
            eprintln!("Failed to load config: {}. Using defaults.", e);
            AppConfig::default()
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections cannot be less than min connections".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Max file size must be greater than 0".to_string());
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
    }
    
    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
            [server]
            host = "0.0.0.0"
            port = 3000
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
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "debug");
    }
    
    #[test]
    fn test_load_invalid_config() {
        let config_str = r#"
            [server]
            host = "localhost"
            port = 0
            timeout_seconds = 30
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_str).unwrap();
        
        let result = load_config(temp_file.path());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port cannot be 0"));
    }
}