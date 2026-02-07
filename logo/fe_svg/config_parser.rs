use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    FileNotFound,
    ParseError(String),
    ValidationError(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => ConfigError::FileNotFound,
                _ => ConfigError::ParseError(format!("Failed to read file: {}", e)),
            })?;

        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(format!(
                    "Invalid format at line {}", line_num + 1
                )));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if key.is_empty() {
                return Err(ConfigError::ParseError(format!(
                    "Empty key at line {}", line_num + 1
                )));
            }

            config.values.insert(key, value);
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), ConfigError> {
        for key in required_keys {
            if !self.values.contains_key(*key) {
                return Err(ConfigError::ValidationError(
                    format!("Missing required key: {}", key)
                ));
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_config() {
        let content = "host=localhost\nport=8080\ntimeout=30\n";
        let config = Config::parse(content).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.len(), 3);
    }

    #[test]
    fn test_parse_with_comments_and_whitespace() {
        let content = "# Server configuration\nhost = localhost\n\nport = 8080  \n# End of config";
        let config = Config::parse(content).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.len(), 2);
    }

    #[test]
    fn test_parse_invalid_format() {
        let content = "host=localhost\ninvalid_line\nport=8080";
        let result = Config::parse(content);
        
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "key1=value1\nkey2=value2").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_validate_required() {
        let content = "required1=value1\nrequired2=value2\noptional=value3";
        let config = Config::parse(content).unwrap();
        
        assert!(config.validate_required(&["required1", "required2"]).is_ok());
        assert!(config.validate_required(&["required1", "missing"]).is_err());
    }

    #[test]
    fn test_get_or_default() {
        let content = "existing=value";
        let config = Config::parse(content).unwrap();
        
        assert_eq!(config.get_or_default("existing", "default"), "value");
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }
}use std::fs;
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
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError("Max connections must be greater than zero".to_string()));
        }
        
        Ok(())
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080
            timeout_seconds = 30

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 20
            pool_timeout_seconds = 10

            [logging]
            level = "info"
            file_path = "/var/log/app.log"
            enable_console = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_invalid_config() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 0
            timeout_seconds = 30

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 20
            pool_timeout_seconds = 10

            [logging]
            level = "info"
            enable_console = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_err());
    }
}