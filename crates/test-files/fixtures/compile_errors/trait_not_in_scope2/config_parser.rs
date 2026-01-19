use std::collections::HashMap;
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

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        config.parse_content(&content)?;
        Ok(config)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if !self.validate_setting(&key, &value) {
                return Err(format!("Invalid value for {}: {}", key, value));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    fn validate_setting(&self, key: &str, value: &str) -> bool {
        match key {
            "timeout" => value.parse::<u32>().is_ok(),
            "retries" => value.parse::<u8>().is_ok(),
            "log_level" => ["debug", "info", "warn", "error"].contains(&value),
            _ => true,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_or_default(&self, key: &str) -> String {
        self.get(key)
            .map(|v| v.clone())
            .unwrap_or_else(|| "".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timeout=60\nretries=5\nlog_level=debug").unwrap();

        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"5".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timeout=invalid").unwrap();

        let result = Config::load_from_file(file.path());
        assert!(result.is_err());
    }
}use std::collections::HashMap;
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
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(separator) = trimmed.find('=') {
                let key = trimmed[..separator].trim().to_string();
                let mut value = trimmed[separator + 1..].trim().to_string();
                
                value = var_pattern.replace_all(&value, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| String::new())
                }).to_string();
                
                self.values.insert(key, value);
            }
        }
        
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_with_default(&self, key: &str, default: &str) -> String {
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
        let config = "host=localhost\nport=8080\ntimeout=30";
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("port"), Some(&"8080".to_string()));
        assert_eq!(parser.get_with_default("timeout", "10"), "30");
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "9090");
        
        let mut parser = ConfigParser::new();
        let config = "server_port=${APP_PORT}\ndebug_mode=true";
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("server_port"), Some(&"9090".to_string()));
    }
}