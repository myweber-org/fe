use std::fs;
use std::collections::HashMap;
use toml::Value;

pub struct Config {
    data: HashMap<String, Value>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            data: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let parsed: Value = content.parse()
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        if let Value::Table(table) = parsed {
            for (key, value) in table {
                self.data.insert(key, value);
            }
            Ok(())
        } else {
            Err("Invalid TOML structure".to_string())
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.data.get(key)
            .and_then(|v| v.as_integer())
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)
            .and_then(|v| v.as_bool())
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.data.get(key)
            .and_then(|v| v.as_float())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
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
        
        let toml_content = r#"
            server_host = "localhost"
            server_port = 8080
            debug_mode = true
            timeout = 30.5
        "#;
        
        write!(temp_file, "{}", toml_content).unwrap();
        
        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(config.get_string("server_host"), Some("localhost".to_string()));
        assert_eq!(config.get_int("server_port"), Some(8080));
        assert_eq!(config.get_bool("debug_mode"), Some(true));
        assert_eq!(config.get_float("timeout"), Some(30.5));
        assert!(config.contains_key("server_host"));
    }

    #[test]
    fn test_missing_key() {
        let config = Config::new();
        assert_eq!(config.get_string("nonexistent"), None);
    }
}