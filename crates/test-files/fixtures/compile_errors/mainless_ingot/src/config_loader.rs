
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse() {
                self.server_port = parsed_port;
            }
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: HashMap<String, String> = serde_json::from_str(&content)?;

        Self::apply_env_overrides(&mut config);

        Ok(Config {
            database_url: config
                .remove("database_url")
                .unwrap_or_else(|| "postgres://localhost:5432".into()),
            server_port: config
                .remove("server_port")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            log_level: config.remove("log_level").unwrap_or_else(|| "info".into()),
            features: config
                .iter()
                .filter(|(k, _)| k.starts_with("feature_"))
                .map(|(k, v)| (k.replace("feature_", ""), v.parse().unwrap_or(false)))
                .collect(),
        })
    }

    fn apply_env_overrides(config: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".into());
        }

        if self.server_port == 0 {
            errors.push("Server port must be greater than 0".into());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!(
                "Invalid log level '{}'. Must be one of: {}",
                self.log_level,
                valid_log_levels.join(", ")
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{
                "database_url": "postgres://localhost:5432/mydb",
                "server_port": "3000",
                "log_level": "debug",
                "feature_caching": "true"
            }}"#
        )
        .unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.features.get("caching"), Some(&true));
    }

    #[test]
    fn test_env_override() {
        env::set_var("APP_DATABASE_URL", "postgres://prod:5432/proddb");
        env::set_var("APP_SERVER_PORT", "8080");

        let mut config_map = HashMap::new();
        config_map.insert("database_url".into(), "default".into());
        config_map.insert("server_port".into(), "3000".into());

        Config::apply_env_overrides(&mut config_map);

        assert_eq!(config_map.get("database_url"), Some(&"postgres://prod:5432/proddb".into()));
        assert_eq!(config_map.get("server_port"), Some(&"8080".into()));

        env::remove_var("APP_DATABASE_URL");
        env::remove_var("APP_SERVER_PORT");
    }
}