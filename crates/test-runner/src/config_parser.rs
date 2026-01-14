use std::collections::HashMap;
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

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if let Ok(num) = value.parse::<f64>() {
                config.numeric_values.insert(key.clone(), num);
            } else if value == "true" || value == "false" {
                let flag = value == "true";
                config.flags.insert(key.clone(), flag);
            } else {
                config.settings.insert(key, value);
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
        self.get_setting(key)
            .map(|s| s.clone())
            .unwrap_or_else(|| default.to_string())
    }
    
    pub fn get_numeric_with_default(&self, key: &str, default: f64) -> f64 {
        self.get_numeric(key).unwrap_or(default)
    }
    
    pub fn get_flag_with_default(&self, key: &str, default: bool) -> bool {
        self.get_flag(key).unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Sample configuration").unwrap();
        writeln!(file, "app_name = MyApplication").unwrap();
        writeln!(file, "max_connections = 100").unwrap();
        writeln!(file, "debug_mode = true").unwrap();
        writeln!(file, "timeout = 30.5").unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        
        assert_eq!(config.get_setting("app_name"), Some(&"MyApplication".to_string()));
        assert_eq!(config.get_numeric("max_connections"), Some(100.0));
        assert_eq!(config.get_flag("debug_mode"), Some(true));
        assert_eq!(config.get_numeric("timeout"), Some(30.5));
        assert_eq!(config.get_setting("nonexistent"), None);
    }
    
    #[test]
    fn test_default_values() {
        let config = Config::new();
        
        assert_eq!(config.get_setting_with_default("missing", "default_value"), "default_value");
        assert_eq!(config.get_numeric_with_default("missing", 42.0), 42.0);
        assert_eq!(config.get_flag_with_default("missing", true), true);
    }
    
    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid_line_without_equals").unwrap();
        
        let result = Config::from_file(file.path());
        assert!(result.is_err());
    }
}