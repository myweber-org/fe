
use std::fs;
use std::collections::HashMap;
use toml::Value;

pub struct Config {
    settings: HashMap<String, Value>,
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
        
        let parsed: Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        if let Value::Table(table) = parsed {
            for (key, value) in table {
                self.settings.insert(key, value);
            }
            Ok(())
        } else {
            Err("Invalid config structure".to_string())
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.settings.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.settings.get(key)
            .and_then(|v| v.as_integer())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.settings.get(key)
            .and_then(|v| v.as_bool())
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.settings.get(key)
            .and_then(|v| v.as_float())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }

    pub fn get_all_keys(&self) -> Vec<&String> {
        self.settings.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let toml_content = r#"
            app_name = "My Application"
            port = 8080
            debug = true
            timeout = 3.5
        "#;
        
        write!(temp_file, "{}", toml_content).unwrap();
        
        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(config.get_string("app_name"), Some("My Application".to_string()));
        assert_eq!(config.get_int("port"), Some(8080));
        assert_eq!(config.get_bool("debug"), Some(true));
        assert_eq!(config.get_float("timeout"), Some(3.5));
        assert!(config.contains_key("app_name"));
    }

    #[test]
    fn test_missing_key() {
        let config = Config::new();
        assert_eq!(config.get_string("nonexistent"), None);
        assert!(!config.contains_key("nonexistent"));
    }
}