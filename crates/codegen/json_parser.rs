use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    version: String,
    settings: HashMap<String, Value>,
}

impl Config {
    fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let config: Config = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(serde::de::Error::custom("Name cannot be empty"));
        }
        
        let version_parts: Vec<&str> = self.version.split('.').collect();
        if version_parts.len() != 3 {
            return Err(serde::de::Error::custom("Version must be in format x.y.z"));
        }

        for part in version_parts {
            if part.parse::<u32>().is_err() {
                return Err(serde::de::Error::custom("Version parts must be numbers"));
            }
        }

        Ok(())
    }

    fn get_setting(&self, key: &str) -> Option<&Value> {
        self.settings.get(key)
    }

    fn merge(&mut self, other: Config) {
        self.settings.extend(other.settings);
    }
}

fn parse_json_string(json_str: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(json_str)?;
    Ok(value)
}

fn create_sample_config() -> Config {
    let mut settings = HashMap::new();
    settings.insert("timeout".to_string(), Value::Number(30.into()));
    settings.insert("debug".to_string(), Value::Bool(true));
    settings.insert("max_connections".to_string(), Value::Number(100.into()));

    Config {
        name: "my_app".to_string(),
        version: "1.2.3".to_string(),
        settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = create_sample_config();
        assert_eq!(config.name, "my_app");
        assert_eq!(config.version, "1.2.3");
        assert!(config.get_setting("debug").is_some());
    }

    #[test]
    fn test_json_parsing() {
        let json_str = r#"{"name": "test", "value": 42}"#;
        let result = parse_json_string(json_str);
        assert!(result.is_ok());
        
        let value = result.unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["value"], 42);
    }

    #[test]
    fn test_config_validation() {
        let mut config = create_sample_config();
        config.name = String::new();
        let validation = config.validate();
        assert!(validation.is_err());
    }
}