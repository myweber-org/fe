use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = self.substitute_env_vars(value.trim());
                self.settings.insert(key.trim().to_string(), processed_value);
            }
        }
        Ok(())
    }

    fn substitute_env_vars(&self, value: &str) -> String {
        let mut result = value.to_string();
        for (key, val) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &val);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        let mut config = Config::new();
        let test_content = "server_port=${APP_PORT}\ndebug_mode=true";
        fs::write("test_config.tmp", test_content).unwrap();

        config.load_from_file("test_config.tmp").unwrap();
        assert_eq!(config.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug_mode"), Some(&"true".to_string()));

        fs::remove_file("test_config.tmp").unwrap();
    }
}