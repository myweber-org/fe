use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub settings: HashMap<String, Value>,
    pub enabled: bool,
}

pub fn parse_json_file(path: &str) -> Result<Config, JsonParseError> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let config: Config = serde_json::from_str(&contents)?;
    
    validate_config(&config)?;
    
    Ok(config)
}

fn validate_config(config: &Config) -> Result<(), JsonParseError> {
    if config.name.is_empty() {
        return Err(JsonParseError::Validation("Name cannot be empty".to_string()));
    }
    
    if config.version.is_empty() {
        return Err(JsonParseError::Validation("Version cannot be empty".to_string()));
    }
    
    let version_parts: Vec<&str> = config.version.split('.').collect();
    if version_parts.len() != 3 {
        return Err(JsonParseError::Validation(
            "Version must be in format X.Y.Z".to_string()
        ));
    }
    
    for part in version_parts {
        if part.parse::<u32>().is_err() {
            return Err(JsonParseError::Validation(
                "Version parts must be numbers".to_string()
            ));
        }
    }
    
    Ok(())
}

pub fn merge_configs(base: &Config, overlay: &Config) -> Config {
    let mut merged = base.clone();
    
    if !overlay.name.is_empty() {
        merged.name = overlay.name.clone();
    }
    
    if !overlay.version.is_empty() {
        merged.version = overlay.version.clone();
    }
    
    merged.enabled = overlay.enabled;
    
    for (key, value) in &overlay.settings {
        merged.settings.insert(key.clone(), value.clone());
    }
    
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json() {
        let json_data = json!({
            "name": "test_app",
            "version": "1.2.3",
            "settings": {
                "timeout": 30,
                "retries": 3
            },
            "enabled": true
        });

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), json_data.to_string()).unwrap();
        
        let result = parse_json_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.name, "test_app");
        assert_eq!(config.version, "1.2.3");
        assert_eq!(config.enabled, true);
    }

    #[test]
    fn test_validation_failure() {
        let json_data = json!({
            "name": "",
            "version": "1.2",
            "settings": {},
            "enabled": true
        });

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), json_data.to_string()).unwrap();
        
        let result = parse_json_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}