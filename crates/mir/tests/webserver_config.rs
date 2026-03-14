use std::env;
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub log_level: String,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            log_level: String::from("info"),
            enable_tls: false,
        }
    }
}

impl ServerConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(file_path)?;
        let config: ServerConfig = toml::from_str(&config_content)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        let mut config = ServerConfig::default();

        if let Ok(host) = env::var("SERVER_HOST") {
            config.host = host;
        }

        if let Ok(port_str) = env::var("SERVER_PORT") {
            if let Ok(port) = port_str.parse() {
                config.port = port;
            }
        }

        if let Ok(max_conn_str) = env::var("MAX_CONNECTIONS") {
            if let Ok(max_conn) = max_conn_str.parse() {
                config.max_connections = max_conn;
            }
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_lowercase();
        }

        if let Ok(enable_tls_str) = env::var("ENABLE_TLS") {
            config.enable_tls = enable_tls_str.to_lowercase() == "true";
        }

        config
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.log_level, "info");
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_env_config() {
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "DEBUG");

        let config = ServerConfig::from_env();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9090);
        assert_eq!(config.log_level, "debug");

        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }

    #[test]
    fn test_address_format() {
        let config = ServerConfig {
            host: String::from("localhost"),
            port: 3000,
            ..Default::default()
        };
        assert_eq!(config.address(), "localhost:3000");
    }
}