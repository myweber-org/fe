use std::collections::HashMap;
use std::fs;
use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
    pub max_connections: u32,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let parsed: Value = content.parse()
            .map_err(|e| format!("Invalid TOML format: {}", e))?;
        
        let database_table = parsed.get("database")
            .ok_or("Missing 'database' section")?
            .as_table()
            .ok_or("'database' section must be a table")?;
        
        let server_table = parsed.get("server")
            .ok_or("Missing 'server' section")?
            .as_table()
            .ok_or("'server' section must be a table")?;
        
        let features_table = parsed.get("features")
            .and_then(|v| v.as_table())
            .unwrap_or(&toml::map::Map::new());
        
        let database = DatabaseConfig {
            host: get_string(database_table, "host")?,
            port: get_u16(database_table, "port")?,
            username: get_string(database_table, "username")?,
            password: get_string(database_table, "password")?,
            database_name: get_string(database_table, "database_name")?,
        };
        
        let server = ServerConfig {
            host: get_string(server_table, "host")?,
            port: get_u16(server_table, "port")?,
            enable_ssl: get_bool(server_table, "enable_ssl").unwrap_or(false),
            max_connections: get_u32(server_table, "max_connections").unwrap_or(100),
        };
        
        let mut features = HashMap::new();
        for (key, value) in features_table {
            if let Some(bool_val) = value.as_bool() {
                features.insert(key.clone(), bool_val);
            }
        }
        
        Ok(Config {
            database,
            server,
            features,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database.port == 0 {
            errors.push("Database port cannot be 0".to_string());
        }
        
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        if self.server.max_connections == 0 {
            errors.push("Max connections must be greater than 0".to_string());
        }
        
        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn get_string(table: &toml::map::Map<String, Value>, key: &str) -> Result<String, String> {
    table.get(key)
        .ok_or(format!("Missing '{}'", key))?
        .as_str()
        .map(|s| s.to_string())
        .ok_or(format!("'{}' must be a string", key))
}

fn get_u16(table: &toml::map::Map<String, Value>, key: &str) -> Result<u16, String> {
    table.get(key)
        .ok_or(format!("Missing '{}'", key))?
        .as_integer()
        .and_then(|n| if n >= 0 && n <= u16::MAX as i64 { Some(n as u16) } else { None })
        .ok_or(format!("'{}' must be a valid u16", key))
}

fn get_u32(table: &toml::map::Map<String, Value>, key: &str) -> Result<u32, String> {
    table.get(key)
        .ok_or(format!("Missing '{}'", key))?
        .as_integer()
        .and_then(|n| if n >= 0 && n <= u32::MAX as i64 { Some(n as u32) } else { None })
        .ok_or(format!("'{}' must be a valid u32", key))
}

fn get_bool(table: &toml::map::Map<String, Value>, key: &str) -> Result<bool, String> {
    table.get(key)
        .ok_or(format!("Missing '{}'", key))?
        .as_bool()
        .ok_or(format!("'{}' must be a boolean", key))
}