use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: HashMap<String, toml::Value> = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        Self::apply_env_overrides(&mut config);

        let database_url = Self::get_string(&config, "database_url")?;
        let server_port = Self::get_u16(&config, "server_port")?;
        let debug_mode = Self::get_bool(&config, "debug_mode").unwrap_or(false);
        let api_keys = Self::get_api_keys(&config)?;

        Ok(Config {
            database_url,
            server_port,
            debug_mode,
            api_keys,
        })
    }

    fn apply_env_overrides(config: &mut HashMap<String, toml::Value>) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.insert("database_url".to_string(), toml::Value::String(db_url));
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.insert("server_port".to_string(), toml::Value::Integer(port_num as i64));
            }
        }
        if let Ok(debug) = env::var("DEBUG_MODE") {
            let debug_bool = debug.to_lowercase() == "true" || debug == "1";
            config.insert("debug_mode".to_string(), toml::Value::Boolean(debug_bool));
        }
    }

    fn get_string(config: &HashMap<String, toml::Value>, key: &str) -> Result<String, String> {
        config.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Missing or invalid string for key: {}", key))
    }

    fn get_u16(config: &HashMap<String, toml::Value>, key: &str) -> Result<u16, String> {
        config.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| u16::try_from(i).ok())
            .ok_or_else(|| format!("Missing or invalid u16 for key: {}", key))
    }

    fn get_bool(config: &HashMap<String, toml::Value>, key: &str) -> Option<bool> {
        config.get(key).and_then(|v| v.as_bool())
    }

    fn get_api_keys(config: &HashMap<String, toml::Value>) -> Result<HashMap<String, String>, String> {
        let mut api_keys = HashMap::new();
        
        if let Some(toml::Value::Table(table)) = config.get("api_keys") {
            for (service, value) in table {
                if let Some(key_str) = value.as_str() {
                    api_keys.insert(service.clone(), key_str.to_string());
                }
            }
        }
        
        Ok(api_keys)
    }
}