use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config, path: &str) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
                tls_enabled: true,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 20,
                timeout_seconds: 30,
            },
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_config(&config, path).unwrap();
        let loaded = load_config(path).unwrap();

        assert_eq!(config.server.host, loaded.server.host);
        assert_eq!(config.server.port, loaded.server.port);
        assert_eq!(config.server.tls_enabled, loaded.server.tls_enabled);
        assert_eq!(config.database.url, loaded.database.url);
        assert_eq!(config.database.max_connections, loaded.database.max_connections);
        assert_eq!(config.database.timeout_seconds, loaded.database.timeout_seconds);
    }
}