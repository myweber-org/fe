use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub features: Vec<String>,
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
            host: "localhost".to_string(),
            port: 8080,
            features: vec!["logging".to_string(), "cache".to_string()],
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        save_config(&config, path).unwrap();
        let loaded = load_config(path).unwrap();

        assert_eq!(config.host, loaded.host);
        assert_eq!(config.port, loaded.port);
        assert_eq!(config.features, loaded.features);
    }
}