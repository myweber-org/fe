use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub cache_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let db_port = env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid DB_PORT: {}", e))?;
        
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid SERVER_PORT: {}", e))?;
        
        let max_conn = env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .map_err(|e| format!("Invalid MAX_CONNECTIONS: {}", e))?;
        
        Ok(Config {
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
                password: env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
                database_name: env::var("DB_NAME").unwrap_or_else(|_| "app_db".to_string()),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: server_port,
                max_connections: max_conn,
                timeout_seconds: env::var("TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse::<u64>()
                    .unwrap_or(30),
            },
            features: FeatureFlags {
                enable_logging: env::var("ENABLE_LOGGING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse::<bool>()
                    .unwrap_or(true),
                enable_metrics: env::var("ENABLE_METRICS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse::<bool>()
                    .unwrap_or(false),
                cache_size: env::var("CACHE_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse::<usize>()
                    .unwrap_or(1000),
            },
        })
    }
    
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        map.insert("DB_HOST".to_string(), self.database.host.clone());
        map.insert("DB_PORT".to_string(), self.database.port.to_string());
        map.insert("DB_USER".to_string(), self.database.username.clone());
        map.insert("DB_PASSWORD".to_string(), self.database.password.clone());
        map.insert("DB_NAME".to_string(), self.database.database_name.clone());
        
        map.insert("SERVER_HOST".to_string(), self.server.host.clone());
        map.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        map.insert("MAX_CONNECTIONS".to_string(), self.server.max_connections.to_string());
        map.insert("TIMEOUT_SECONDS".to_string(), self.server.timeout_seconds.to_string());
        
        map.insert("ENABLE_LOGGING".to_string(), self.features.enable_logging.to_string());
        map.insert("ENABLE_METRICS".to_string(), self.features.enable_metrics.to_string());
        map.insert("CACHE_SIZE".to_string(), self.features.cache_size.to_string());
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_from_env() {
        env::set_var("DB_HOST", "test_host");
        env::set_var("DB_PORT", "3306");
        env::set_var("SERVER_PORT", "9090");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.database.host, "test_host");
        assert_eq!(config.database.port, 3306);
        assert_eq!(config.server.port, 9090);
        
        env::remove_var("DB_HOST");
        env::remove_var("DB_PORT");
        env::remove_var("SERVER_PORT");
    }
    
    #[test]
    fn test_config_defaults() {
        env::remove_var("DB_HOST");
        env::remove_var("DB_PORT");
        env::remove_var("SERVER_PORT");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
    }
}
use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        let var_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                for var_cap in var_re.captures_iter(&value) {
                    let var_name = &var_cap[1];
                    if let Ok(env_value) = env::var(var_name) {
                        value = value.replace(&var_cap[0], &env_value);
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = "HOST=localhost\nPORT=8080\nDEBUG=true";
        parser.load_from_str(config).unwrap();

        assert_eq!(parser.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        
        let mut parser = ConfigParser::new();
        let config = "PORT=${APP_PORT}\nHOST=127.0.0.1";
        parser.load_from_str(config).unwrap();

        assert_eq!(parser.get("PORT"), Some(&"3000".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = "INVALID LINE";
        let result = parser.load_from_str(config);
        
        assert!(result.is_err());
    }
}