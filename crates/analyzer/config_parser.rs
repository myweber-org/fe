
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

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();
        let mut in_env_var = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                in_env_var = true;
                var_name.clear();
                continue;
            }

            if in_env_var {
                if ch == '}' {
                    let env_value = env::var(&var_name).unwrap_or_default();
                    result.push_str(&env_value);
                    in_env_var = false;
                } else {
                    var_name.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        if in_env_var {
            result.push_str("${");
            result.push_str(&var_name);
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
        env::set_var("APP_USER", "rustacean");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "USER=${APP_USER}").unwrap();
        writeln!(file, "PATH=/home/${APP_USER}/data").unwrap();
        writeln!(file, "MISSING=${UNDEFINED_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("USER"), Some(&"rustacean".to_string()));
        assert_eq!(config.get("PATH"), Some(&"/home/rustacean/data".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"".to_string()));
    }
}