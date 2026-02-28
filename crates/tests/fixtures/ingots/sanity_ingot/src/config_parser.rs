use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

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
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
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
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "URL=postgres://user:${DB_PASSWORD}@localhost").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("URL"), Some(&"postgres://user:secret123@localhost".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_files: usize,
    pub max_file_size_mb: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub feature_flags: Vec<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database_name: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "0.0.0.0".to_string(),
            port: 8080,
            enable_https: false,
            cert_path: None,
            key_path: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            file_path: "logs/app.log".to_string(),
            max_files: 5,
            max_file_size_mb: 10,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            feature_flags: vec![],
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_str)?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn from_file_with_defaults<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let default_config = AppConfig::default();
        
        if !path.as_ref().exists() {
            return Ok(default_config);
        }
        
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_str)?;
        
        config.database = Self::merge_database_config(&default_config.database, &config.database);
        config.server = Self::merge_server_config(&default_config.server, &config.server);
        config.logging = Self::merge_logging_config(&default_config.logging, &config.logging);
        
        config.validate()?;
        Ok(config)
    }
    
    fn merge_database_config(default: &DatabaseConfig, provided: &DatabaseConfig) -> DatabaseConfig {
        DatabaseConfig {
            host: if provided.host.is_empty() { default.host.clone() } else { provided.host.clone() },
            port: if provided.port == 0 { default.port } else { provided.port },
            username: if provided.username.is_empty() { default.username.clone() } else { provided.username.clone() },
            password: provided.password.clone(),
            database_name: if provided.database_name.is_empty() { default.database_name.clone() } else { provided.database_name.clone() },
            pool_size: if provided.pool_size == 0 { default.pool_size } else { provided.pool_size },
        }
    }
    
    fn merge_server_config(default: &ServerConfig, provided: &ServerConfig) -> ServerConfig {
        ServerConfig {
            address: if provided.address.is_empty() { default.address.clone() } else { provided.address.clone() },
            port: if provided.port == 0 { default.port } else { provided.port },
            enable_https: provided.enable_https,
            cert_path: provided.cert_path.clone().or_else(|| default.cert_path.clone()),
            key_path: provided.key_path.clone().or_else(|| default.key_path.clone()),
        }
    }
    
    fn merge_logging_config(default: &LoggingConfig, provided: &LoggingConfig) -> LoggingConfig {
        LoggingConfig {
            level: if provided.level.is_empty() { default.level.clone() } else { provided.level.clone() },
            file_path: if provided.file_path.is_empty() { default.file_path.clone() } else { provided.file_path.clone() },
            max_files: if provided.max_files == 0 { default.max_files } else { provided.max_files },
            max_file_size_mb: if provided.max_file_size_mb == 0 { default.max_file_size_mb } else { provided.max_file_size_mb },
        }
    }
    
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database.port == 0 {
            return Err("Database port cannot be 0".into());
        }
        
        if self.database.host.is_empty() {
            return Err("Database host cannot be empty".into());
        }
        
        if self.database.database_name.is_empty() {
            return Err("Database name cannot be empty".into());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be 0".into());
        }
        
        if self.server.enable_https {
            if self.server.cert_path.is_none() || self.server.key_path.is_none() {
                return Err("HTTPS requires both certificate and key paths".into());
            }
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("Invalid log level: {}. Valid levels are: {:?}", 
                self.logging.level, valid_log_levels).into());
        }
        
        Ok(())
    }
    
    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(self)?)
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = self.to_yaml()?;
        fs::write(path, yaml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_config_from_file() {
        let yaml_content = r#"
database:
  host: "db.example.com"
  port: 5433
  username: "app_user"
  password: "secure_password"
  database_name: "production_db"
  pool_size: 20
server:
  address: "127.0.0.1"
  port: 8443
  enable_https: true
  cert_path: "/path/to/cert.pem"
  key_path: "/path/to/key.pem"
logging:
  level: "debug"
  file_path: "/var/log/app.log"
  max_files: 10
  max_file_size_mb: 50
feature_flags:
  - "new_ui"
  - "beta_features"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), yaml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "db.example.com");
        assert_eq!(config.database.port, 5433);
        assert_eq!(config.server.port, 8443);
        assert!(config.server.enable_https);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.feature_flags.len(), 2);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.database.port = 0;
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_merge_with_defaults() {
        let partial_yaml = r#"
database:
  host: "custom_host"
  port: 9999
server:
  enable_https: true
logging:
  level: "warn"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), partial_yaml).unwrap();
        
        let config = AppConfig::from_file_with_defaults(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "custom_host");
        assert_eq!(config.database.port, 9999);
        assert_eq!(config.database.username, "postgres");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_https);
        assert_eq!(config.logging.level, "warn");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
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
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "NONEXISTENT=$MISSING_VAR").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NONEXISTENT"), Some(&"$MISSING_VAR".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}