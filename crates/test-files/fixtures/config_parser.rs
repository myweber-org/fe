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
            Err("Invalid TOML structure".to_string())
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

    pub fn get_array(&self, key: &str) -> Option<Vec<String>> {
        self.settings.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
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
            enable_logging = true
            allowed_users = ["user1", "user2"]
        "#;
        
        write!(temp_file, "{}", toml_content).unwrap();
        
        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(config.get_string("server_host"), Some("localhost".to_string()));
        assert_eq!(config.get_int("server_port"), Some(8080));
        assert_eq!(config.get_bool("enable_logging"), Some(true));
        assert_eq!(config.get_array("allowed_users"), 
                   Some(vec!["user1".to_string(), "user2".to_string()]));
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
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

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
        writeln!(file, "  TIMEOUT = 30  ").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "URL=postgres://user:${DB_PASSWORD}@localhost").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("URL"), Some(&"postgres://user:secret123@localhost".to_string()));
    }
}