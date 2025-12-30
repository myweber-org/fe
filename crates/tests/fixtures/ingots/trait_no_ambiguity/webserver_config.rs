use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub thread_pool_size: usize,
    pub request_timeout_seconds: u64,
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            thread_pool_size: 4,
            request_timeout_seconds: 30,
            enable_logging: true,
        }
    }
}

impl ServerConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: ServerConfig = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.port == 0 {
            return Err("Port cannot be zero");
        }
        if self.thread_pool_size == 0 {
            return Err("Thread pool size must be greater than zero");
        }
        Ok(())
    }
}