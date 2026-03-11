use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub cache_size: usize,
    pub features: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app"),
            cache_size: 100,
            features: vec![String::from("logging"), String::from("metrics")],
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::default();
        let parsed: HashMap<String, String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        if let Some(port_str) = parsed.get("server_port") {
            config.server_port = port_str
                .parse()
                .map_err(|_| format!("Invalid port number: {}", port_str))?;
        }

        if let Some(db_url) = parsed.get("database_url") {
            config.database_url = db_url.clone();
        }

        if let Some(cache_str) = parsed.get("cache_size") {
            config.cache_size = cache_str
                .parse()
                .map_err(|_| format!("Invalid cache size: {}", cache_str))?;
        }

        if let Some(features_str) = parsed.get("features") {
            config.features = features_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        if self.cache_size == 0 {
            return Err(String::from("Cache size cannot be zero"));
        }
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }
        Ok(())
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("server_port={}\n", self.server_port));
        output.push_str(&format!("database_url={}\n", self.database_url));
        output.push_str(&format!("cache_size={}\n", self.cache_size));
        output.push_str(&format!("features={}\n", self.features.join(",")));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgresql://localhost:5432/app");
        assert_eq!(config.cache_size, 100);
        assert!(config.features.contains(&String::from("logging")));
    }

    #[test]
    fn test_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "server_port=9000").unwrap();
        writeln!(file, "database_url=postgresql://prod:5432/db").unwrap();
        writeln!(file, "cache_size=500").unwrap();
        writeln!(file, "features=auth,api,cache").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.database_url, "postgresql://prod:5432/db");
        assert_eq!(config.cache_size, 500);
        assert_eq!(config.features, vec!["auth", "api", "cache"]);
    }

    #[test]
    fn test_validation() {
        let mut config = Config::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_to_string() {
        let config = Config::default();
        let serialized = config.to_string();
        assert!(serialized.contains("server_port=8080"));
        assert!(serialized.contains("database_url=postgresql://localhost:5432/app"));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                let env_value = env::var(&var_name).unwrap_or_default();
                result.push_str(&env_value);
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${{DB_PASSWORD}}").unwrap();
        writeln!(file, "NESTED=prefix_${{DB_PASSWORD}}_suffix").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DB_PASS"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NESTED"), Some(&"prefix_secret123_suffix".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}