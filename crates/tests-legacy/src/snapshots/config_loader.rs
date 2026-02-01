
use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
    pub feature_flags: HashMap<String, bool>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set".to_string())?;
        
        let api_key = env::var("API_KEY")
            .map_err(|_| "API_KEY environment variable not set".to_string())?;
        
        let max_connections = env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid MAX_CONNECTIONS value: {}", e))?;
        
        let debug_mode = env::var("DEBUG_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        let mut feature_flags = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let flag_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.parse::<bool>().unwrap_or(false);
                feature_flags.insert(flag_name, enabled);
            }
        }
        
        Ok(Config {
            database_url,
            api_key,
            max_connections,
            debug_mode,
            feature_flags,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if self.api_key.len() < 32 {
            errors.push("API key must be at least 32 characters".to_string());
        }
        
        if self.max_connections == 0 {
            errors.push("Max connections must be greater than 0".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("API_KEY", "test_key_that_is_long_enough_for_validation");
        env::set_var("FEATURE_NEW_API", "true");
        
        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert!(config.is_feature_enabled("new_api"));
    }
    
    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            api_key: "short".to_string(),
            max_connections: 0,
            debug_mode: false,
            feature_flags: HashMap::new(),
        };
        
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 3);
    }
}