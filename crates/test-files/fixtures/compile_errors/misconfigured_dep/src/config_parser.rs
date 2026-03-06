use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub cache_size: usize,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let lines: Vec<&str> = content.lines().collect();
        let mut config_map = HashMap::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            config_map.insert(key, value);
        }
        
        Self::from_map(config_map)
    }
    
    fn from_map(map: HashMap<String, String>) -> Result<Self, String> {
        let server_port = map.get("server_port")
            .ok_or("Missing 'server_port' in config")?
            .parse::<u16>()
            .map_err(|e| format!("Invalid server_port: {}", e))?;
        
        let database_url = map.get("database_url")
            .ok_or("Missing 'database_url' in config")?
            .to_string();
        
        let cache_size = map.get("cache_size")
            .map(|s| s.parse::<usize>().unwrap_or(1024))
            .unwrap_or(1024);
        
        let features = map.get("features")
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);
        
        let mut metadata = HashMap::new();
        for (key, value) in map {
            if key.starts_with("meta.") {
                let meta_key = key.trim_start_matches("meta.").to_string();
                metadata.insert(meta_key, value);
            }
        }
        
        Ok(Config {
            server_port,
            database_url,
            cache_size,
            features,
            metadata,
        })
    }
    
    pub fn default() -> Self {
        Config {
            server_port: 8080,
            database_url: "postgresql://localhost:5432/appdb".to_string(),
            cache_size: 1024,
            features: vec!["logging".to_string(), "metrics".to_string()],
            metadata: HashMap::new(),
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("server_port cannot be 0".to_string());
        }
        
        if self.database_url.is_empty() {
            return Err("database_url cannot be empty".to_string());
        }
        
        if self.cache_size > 100_000 {
            return Err("cache_size cannot exceed 100,000".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "server_port=3000").unwrap();
        writeln!(temp_file, "database_url=postgresql://prod:5432/db").unwrap();
        writeln!(temp_file, "cache_size=2048").unwrap();
        writeln!(temp_file, "features=auth,caching,api").unwrap();
        writeln!(temp_file, "meta.environment=production").unwrap();
        
        let config = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://prod:5432/db");
        assert_eq!(config.cache_size, 2048);
        assert_eq!(config.features, vec!["auth", "caching", "api"]);
        assert_eq!(config.metadata.get("environment"), Some(&"production".to_string()));
    }
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgresql://localhost:5432/appdb");
        assert_eq!(config.cache_size, 1024);
        assert!(config.features.contains(&"logging".to_string()));
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.server_port = 0;
        assert!(config.validate().is_err());
        
        config.server_port = 8080;
        config.database_url = String::new();
        assert!(config.validate().is_err());
        
        config.database_url = "valid".to_string();
        config.cache_size = 200_000;
        assert!(config.validate().is_err());
    }
}