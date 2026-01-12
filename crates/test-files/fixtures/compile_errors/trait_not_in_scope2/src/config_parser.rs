use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();
                
                if value.starts_with('$') {
                    let var_name = &value[1..];
                    if let Ok(env_value) = env::var(var_name) {
                        value = env_value;
                    }
                }
                
                values.insert(key, value);
            }
        }
        
        Ok(Config { values })
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/mydb").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();
        
        env::set_var("SECRET_KEY", "abc123");
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/mydb");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert_eq!(config.get("API_KEY").unwrap(), "abc123");
        assert_eq!(config.get("NON_EXISTENT"), None);
        assert_eq!(config.get_or_default("NON_EXISTENT", "default"), "default");
    }
}