
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut parsed = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::substitute_env_vars(&raw_value);
            parsed.insert(key, value);
        }

        Ok(Config {
            database_url: parsed
                .get("DATABASE_URL")
                .ok_or("Missing DATABASE_URL")?
                .clone(),
            api_key: parsed
                .get("API_KEY")
                .ok_or("Missing API_KEY")?
                .clone(),
            max_connections: parsed
                .get("MAX_CONNECTIONS")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            debug_mode: parsed
                .get("DEBUG_MODE")
                .map(|v| v == "true")
                .unwrap_or(false),
        })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &env_value);
        }
        result
    }
}