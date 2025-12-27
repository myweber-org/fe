use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config.apply_setting(key, value);
            }
        }
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_setting(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = value.to_string(),
            "SERVER_PORT" => {
                if let Ok(port) = value.parse() {
                    self.server_port = port;
                }
            }
            "LOG_LEVEL" => self.log_level = value.to_string(),
            _ if key.starts_with("FEATURE_") => {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                self.features.insert(feature_name, enabled);
            }
            _ => {}
        }
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                self.features.insert(feature_name, enabled);
            }
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from("postgresql://localhost:5432/app"),
            server_port: 8080,
            log_level: String::from("info"),
            features: HashMap::new(),
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
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgresql://db:5432/test").unwrap();
        writeln!(config_file, "SERVER_PORT=9000").unwrap();
        writeln!(config_file, "# This is a comment").unwrap();
        writeln!(config_file, "FEATURE_API_V2=true").unwrap();
        writeln!(config_file, "FEATURE_LEGACY_SUPPORT=false").unwrap();
        
        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgresql://db:5432/test");
        assert_eq!(config.server_port, 9000);
        assert!(config.is_feature_enabled("api_v2"));
        assert!(!config.is_feature_enabled("legacy_support"));
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("DATABASE_URL", "postgresql://env:5432/override");
        env::set_var("FEATURE_EXPERIMENTAL", "true");
        
        let config = Config::default();
        assert_eq!(config.database_url, "postgresql://env:5432/override");
        assert!(config.is_feature_enabled("experimental"));
        
        env::remove_var("DATABASE_URL");
        env::remove_var("FEATURE_EXPERIMENTAL");
    }
}