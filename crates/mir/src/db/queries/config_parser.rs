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
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s*$").unwrap();
        let env_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                for env_cap in env_re.captures_iter(&value) {
                    let env_var = &env_cap[1];
                    if let Ok(env_value) = env::var(env_var) {
                        value = value.replace(&format!("${{{}}}", env_var), &env_value);
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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

        parser.load_from_str(config).unwrap();
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        env::set_var("DB_HOST", "postgres");

        let mut parser = ConfigParser::new();
        let config = r#"
            port = ${APP_PORT}
            database = ${DB_HOST}
            combined = server-${DB_HOST}:${APP_PORT}
        "#;

        parser.load_from_str(config).unwrap();
        assert_eq!(parser.get("port"), Some(&"3000".to_string()));
        assert_eq!(parser.get("database"), Some(&"postgres".to_string()));
        assert_eq!(parser.get("combined"), Some(&"server-postgres:3000".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = r#"
            valid_key = value
            invalid line without equals
            another_valid = value2
        "#;

        let result = parser.load_from_str(config);
        assert!(result.is_err());
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

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
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
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size: usize,
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
                connect_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("./logs/app.log".to_string()),
                max_file_size: 10 * 1024 * 1024,
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

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(AppConfig::default())
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
    
    if config.database.max_connections == 0 {
        return Err("Max connections must be greater than 0".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(path, toml_string)?;
    Ok(())
}use std::fs;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config.settings.insert(key, value);
            } else {
                return Err(format!("Invalid config line: {}", line).into());
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        for key in keys {
            if !self.settings.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "HOST=localhost").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("HOST".to_string(), "localhost".to_string());
        config.settings.insert("PORT".to_string(), "8080".to_string());

        let result = config.validate_required(&["HOST", "PORT", "API_KEY"]);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["API_KEY"]);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub numeric_values: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            numeric_values: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
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

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            match current_section.as_str() {
                "settings" => {
                    config.settings.insert(key, value);
                }
                "numeric" => {
                    let num_value = value.parse::<f64>()
                        .map_err(|_| format!("Invalid numeric value at line {}", line_num + 1))?;
                    config.numeric_values.insert(key, num_value);
                }
                "flags" => {
                    let bool_value = match value.to_lowercase().as_str() {
                        "true" | "1" | "yes" => true,
                        "false" | "0" | "no" => false,
                        _ => return Err(format!("Invalid boolean value at line {}", line_num + 1)),
                    };
                    config.flags.insert(key, bool_value);
                }
                _ => {
                    config.settings.insert(key, value);
                }
            }
        }

        Ok(config)
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }

    pub fn get_flag(&self, key: &str) -> Option<bool> {
        self.flags.get(key).copied()
    }

    pub fn get_setting_with_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn get_numeric_with_default(&self, key: &str, default: f64) -> f64 {
        self.numeric_values.get(key)
            .copied()
            .unwrap_or(default)
    }

    pub fn get_flag_with_default(&self, key: &str, default: bool) -> bool {
        self.flags.get(key)
            .copied()
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
# Sample configuration
[settings]
server_host = localhost
server_port = 8080

[numeric]
timeout = 30.5
retry_count = 3.0

[flags]
enable_logging = true
debug_mode = false
"#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = Config::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.get_setting("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_numeric("timeout"), Some(30.5));
        assert_eq!(config.get_flag("enable_logging"), Some(true));
        assert_eq!(config.get_flag_with_default("nonexistent", false), false);
    }

    #[test]
    fn test_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_content = "invalid_line_without_equals";
        write!(temp_file, "{}", invalid_content).unwrap();

        let result = Config::load_from_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::collections::HashMap;
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
                return Err(format!("Invalid line format: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_env_vars(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn resolve_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
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
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
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
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_ENV", "production");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENVIRONMENT=${{APP_ENV}}").unwrap();
        writeln!(file, "PATH=/home/${{USER}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        
        if let Ok(user) = env::var("USER") {
            assert_eq!(config.get("PATH"), Some(&format!("/home/{}/data", user)));
        }
    }
}