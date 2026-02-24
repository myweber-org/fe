use std::collections::HashMap;
use std::env;
use std::fs;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = self.resolve_value(value.trim());
                self.values.insert(key, value);
            }
        }

        Ok(())
    }

    fn resolve_value(&self, raw_value: &str) -> String {
        if raw_value.starts_with('$') {
            let var_name = &raw_value[1..];
            env::var(var_name).unwrap_or_else(|_| raw_value.to_string())
        } else {
            raw_value.to_string()
        }
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
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "HOST=localhost\nPORT=8080\n").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "PASSWORD=$DB_PASSWORD\n").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
    }
}