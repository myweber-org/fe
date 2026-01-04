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
}