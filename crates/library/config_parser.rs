use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_logging: bool,
    pub allowed_hosts: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                settings.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_hashmap(&settings)
    }

    fn from_hashmap(map: &HashMap<String, String>) -> Result<Self, String> {
        let server_address = map
            .get("server_address")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = map
            .get("port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let max_connections = map
            .get("max_connections")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let enable_logging = map
            .get("enable_logging")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true);

        let allowed_hosts = map
            .get("allowed_hosts")
            .map(|s| s.split(',').map(|h| h.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["localhost".to_string()]);

        if port == 0 || port > 65535 {
            return Err(format!("Invalid port number: {}", port));
        }

        if max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        Ok(Config {
            server_address,
            port,
            max_connections,
            enable_logging,
            allowed_hosts,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_address.is_empty() {
            return Err("server_address cannot be empty".to_string());
        }

        if self.allowed_hosts.is_empty() {
            return Err("allowed_hosts cannot be empty".to_string());
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_logging: true,
            allowed_hosts: vec!["localhost".to_string()],
        }
    }
}