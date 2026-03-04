
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
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_PORT", "8080");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${APP_PORT}").unwrap();
        writeln!(file, "HOST=localhost:${APP_PORT}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
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

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
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
        writeln!(file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/test");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(!config.contains_key("NONEXISTENT"));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY=$API_KEY").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("KEY").unwrap(), "secret123");
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
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_env_var = false;
        let mut env_var_name = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                in_env_var = true;
                env_var_name.clear();
                continue;
            }

            if in_env_var {
                if ch == '}' {
                    let env_value = env::var(&env_var_name).unwrap_or_default();
                    result.push_str(&env_value);
                    in_env_var = false;
                } else {
                    env_var_name.push(ch);
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
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/home/user/${{USER}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"mysecret123".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, PartialEq)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                pool_size: 10,
                timeout_secs: 30,
            },
            logging: LoggingConfig {
                level: "INFO".to_string(),
                file_path: None,
                enable_console: true,
            },
        }
    }
}

pub fn parse_config_file(path: &str) -> Result<Config, io::Error> {
    let content = fs::read_to_string(path)?;
    parse_config(&content)
}

pub fn parse_config(content: &str) -> Result<Config, io::Error> {
    let mut config = Config::default();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_section = String::new();
    let mut section_map: HashMap<String, Vec<(&str, &str)>> = HashMap::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }

        if let Some(pos) = trimmed.find('=') {
            let key = trimmed[..pos].trim();
            let value = trimmed[pos + 1..].trim();
            section_map
                .entry(current_section.clone())
                .or_insert_with(Vec::new)
                .push((key, value));
        }
    }

    if let Some(server_entries) = section_map.get("server") {
        for (key, value) in server_entries {
            match *key {
                "host" => config.server.host = value.to_string(),
                "port" => {
                    if let Ok(port) = value.parse() {
                        config.server.port = port;
                    }
                }
                "enable_ssl" => {
                    config.server.enable_ssl = value.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
    }

    if let Some(db_entries) = section_map.get("database") {
        for (key, value) in db_entries {
            match *key {
                "url" => config.database.url = value.to_string(),
                "pool_size" => {
                    if let Ok(size) = value.parse() {
                        config.database.pool_size = size;
                    }
                }
                "timeout_secs" => {
                    if let Ok(timeout) = value.parse() {
                        config.database.timeout_secs = timeout;
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(log_entries) = section_map.get("logging") {
        for (key, value) in log_entries {
            match *key {
                "level" => config.logging.level = value.to_string(),
                "file_path" => {
                    if value != "null" {
                        config.logging.file_path = Some(value.to_string());
                    }
                }
                "enable_console" => {
                    config.logging.enable_console = value.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert!(!config.server.enable_ssl);
        assert_eq!(config.database.url, "postgresql://localhost:5432/mydb");
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.database.timeout_secs, 30);
        assert_eq!(config.logging.level, "INFO");
        assert!(config.logging.enable_console);
    }

    #[test]
    fn test_parse_config() {
        let content = r#"
[server]
host = 0.0.0.0
port = 9000
enable_ssl = true

[database]
url = postgresql://prod-db:5432/production
pool_size = 20
timeout_secs = 60

[logging]
level = DEBUG
file_path = /var/log/app.log
enable_console = false
"#;

        let config = parse_config(content).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert!(config.server.enable_ssl);
        assert_eq!(config.database.url, "postgresql://prod-db:5432/production");
        assert_eq!(config.database.pool_size, 20);
        assert_eq!(config.database.timeout_secs, 60);
        assert_eq!(config.logging.level, "DEBUG");
        assert_eq!(config.logging.file_path, Some("/var/log/app.log".to_string()));
        assert!(!config.logging.enable_console);
    }

    #[test]
    fn test_parse_config_with_comments_and_whitespace() {
        let content = r#"
# Server configuration
[server]
host = 192.168.1.100  # Bind address
port = 443

# Database settings
[database]
url = mysql://localhost:3306/testdb

[logging]
level = WARN
file_path = null
enable_console = true
"#;

        let config = parse_config(content).unwrap();
        assert_eq!(config.server.host, "192.168.1.100");
        assert_eq!(config.server.port, 443);
        assert_eq!(config.database.url, "mysql://localhost:3306/testdb");
        assert_eq!(config.logging.level, "WARN");
        assert_eq!(config.logging.file_path, None);
        assert!(config.logging.enable_console);
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.parse_content(&content)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        for key in required_keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
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
    fn test_config_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "server=localhost\nport=8080\n# This is a comment\n\ntimeout=60"
        )
        .unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("server"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
        assert_eq!(config.get_with_default("nonexistent", "fallback"), "fallback");
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        let required = vec!["server", "port", "log_level"];
        let result = config.validate_required(&required);
        assert!(result.is_err());
        
        config.settings.insert("server".to_string(), "localhost".to_string());
        config.settings.insert("port".to_string(), "8080".to_string());
        let result = config.validate_required(&required);
        assert!(result.is_ok());
    }
}