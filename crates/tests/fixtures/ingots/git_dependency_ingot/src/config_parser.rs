use std::collections::HashMap;
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

impl Config {
    pub fn new() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432/mydb"),
            max_connections: 10,
            timeout_seconds: 30,
            features: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
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

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos+1..].trim().to_string();
                
                config.parse_field(&current_section, &key, &value)
                    .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
            } else {
                return Err(format!("Line {}: Invalid format, expected key=value", line_num + 1));
            }
        }

        config.validate()?;
        Ok(config)
    }

    fn parse_field(&mut self, section: &str, key: &str, value: &str) -> Result<(), String> {
        match (section, key) {
            ("database", "url") => {
                if value.is_empty() {
                    return Err("Database URL cannot be empty".to_string());
                }
                self.database_url = value.to_string();
            }
            ("database", "max_connections") => {
                self.max_connections = value.parse()
                    .map_err(|_| "max_connections must be a positive integer".to_string())?;
            }
            ("connection", "timeout") => {
                self.timeout_seconds = value.parse()
                    .map_err(|_| "timeout must be a positive integer".to_string())?;
            }
            ("features", _) => {
                self.features.push(value.to_string());
            }
            ("metadata", _) => {
                self.metadata.insert(key.to_string(), value.to_string());
            }
            _ => return Err(format!("Unknown configuration key: {}.{}", section, key)),
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }
        
        if self.timeout_seconds == 0 {
            return Err("timeout must be greater than 0".to_string());
        }

        if !self.database_url.starts_with("postgresql://") {
            return Err("Only PostgreSQL database URLs are supported".to_string());
        }

        Ok(())
    }

    pub fn get_feature(&self, feature: &str) -> bool {
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
        let config = Config::new();
        assert_eq!(config.database_url, "postgresql://localhost:5432/mydb");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_valid_config_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = postgresql://localhost:5432/production").unwrap();
        writeln!(file, "max_connections = 20").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "# Connection settings").unwrap();
        writeln!(file, "[connection]").unwrap();
        writeln!(file, "timeout = 60").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "[features]").unwrap();
        writeln!(file, "logging = true").unwrap();
        writeln!(file, "caching = enabled").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "[metadata]").unwrap();
        writeln!(file, "version = 1.0.0").unwrap();
        writeln!(file, "environment = production").unwrap();

        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://localhost:5432/production");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.get_feature("logging"));
        assert!(config.get_feature("caching"));
        assert_eq!(config.get_metadata("version"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get_metadata("environment"), Some(&"production".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[database]").unwrap();
        writeln!(file, "url = ").unwrap();
        
        let result = Config::from_file(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Database URL cannot be empty"));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.max_connections = 0;
        assert!(config.validate().is_err());
        
        config.max_connections = 10;
        config.timeout_seconds = 0;
        assert!(config.validate().is_err());
        
        config.timeout_seconds = 30;
        config.database_url = "mysql://localhost:3306/test".to_string();
        assert!(config.validate().is_err());
    }
}