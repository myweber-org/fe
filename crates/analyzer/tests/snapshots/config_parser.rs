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
        
        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let raw_value = caps[2].to_string();
                let processed_value = self.process_value(&raw_value)?;
                self.values.insert(key, processed_value);
            } else {
                return Err(format!("Invalid line format: {}", line));
            }
        }
        
        Ok(())
    }

    fn process_value(&self, value: &str) -> Result<String, String> {
        let env_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = value.to_string();
        
        for caps in env_regex.captures_iter(value) {
            let env_var = &caps[1];
            match env::var(env_var) {
                Ok(env_value) => {
                    result = result.replace(&caps[0], &env_value);
                }
                Err(_) => {
                    return Err(format!("Environment variable '{}' not found", env_var));
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
        let config = "APP_NAME=MyApplication\nVERSION=1.0.0\nDEBUG=true";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("APP_NAME"), Some(&"MyApplication".to_string()));
        assert_eq!(parser.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut parser = ConfigParser::new();
        let config = "DATABASE_URL=${DB_HOST}:5432";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("DATABASE_URL"), Some(&"localhost:5432".to_string()));
    }
}
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

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // Skip '}'
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
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
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
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
        assert_eq!(config.get("HOST").unwrap(), "localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("MISSING").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/opt/app/${{APP_SECRET}}/data").unwrap();
        writeln!(file, "MISSING=${{UNDEFINED_VAR}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET").unwrap(), "mysecret123");
        assert_eq!(config.get("PATH").unwrap(), "/opt/app/mysecret123/data");
        assert_eq!(config.get("MISSING").unwrap(), "${UNDEFINED_VAR}");
    }
}use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                self.settings.insert(key, value);
            } else {
                return Err(format!("Invalid config line: {}", trimmed));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost\nport=8080\n# This is a comment").unwrap();

        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_get_or_default() {
        let mut config = Config::new();
        config.settings.insert("timeout".to_string(), "30".to_string());

        assert_eq!(config.get_or_default("timeout", "10"), "30");
        assert_eq!(config.get_or_default("retries", "3"), "3");
    }

    #[test]
    fn test_invalid_config_line() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}use std::fs;
use std::collections::HashMap;
use serde::Deserialize;

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
    pub enable_ssl: bool,
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

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        vars.insert("DB_URL".to_string(), self.database.url.clone());
        vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        vars
    }
}

pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }
    if config.database.max_connections == 0 {
        return Err("Database max connections must be positive".to_string());
    }
    if !["error", "warn", "info", "debug", "trace"].contains(&config.logging.level.as_str()) {
        return Err("Invalid log level specified".to_string());
    }
    Ok(())
}