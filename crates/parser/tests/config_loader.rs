use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: HashMap<String, String> = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        Self::override_with_env(&mut config);
        
        let database_url = config
            .remove("database_url")
            .ok_or_else(|| "Missing database_url in config".to_string())?;
        
        let server_port = config
            .remove("server_port")
            .ok_or_else(|| "Missing server_port in config".to_string())?
            .parse::<u16>()
            .map_err(|e| format!("Invalid server_port: {}", e))?;
        
        let debug_mode = config
            .remove("debug_mode")
            .unwrap_or_else(|| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        let api_keys = config
            .into_iter()
            .filter(|(k, _)| k.starts_with("api_key_"))
            .map(|(k, v)| (k.replace("api_key_", ""), v))
            .collect();
        
        Ok(Config {
            database_url,
            server_port,
            debug_mode,
            api_keys,
        })
    }
    
    fn override_with_env(config: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }
    }
    
    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        self.api_keys.get(service)
    }
}