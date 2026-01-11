use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;

        let parsed: HashMap<String, toml::Value> = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Self::validate_and_build(parsed)
    }

    fn validate_and_build(data: HashMap<String, toml::Value>) -> Result<Self, ConfigError> {
        let database = Self::parse_database(&data)?;
        let server = Self::parse_server(&data)?;
        let features = Self::parse_features(&data);

        Ok(Config {
            database,
            server,
            features,
        })
    }

    fn parse_database(data: &HashMap<String, toml::Value>) -> Result<DatabaseConfig, ConfigError> {
        let db_table = data.get("database")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing database section".to_string()))?;

        let host = db_table.get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.host".to_string()))?
            .to_string();

        let port = db_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.port".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError("Invalid database port".to_string()));
        }

        let username = db_table.get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.username".to_string()))?
            .to_string();

        let password = db_table.get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.password".to_string()))?
            .to_string();

        Ok(DatabaseConfig {
            host,
            port: port as u16,
            username,
            password,
        })
    }

    fn parse_server(data: &HashMap<String, toml::Value>) -> Result<ServerConfig, ConfigError> {
        let server_table = data.get("server")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing server section".to_string()))?;

        let host = server_table.get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing server.host".to_string()))?
            .to_string();

        let port = server_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Missing server.port".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError("Invalid server port".to_string()));
        }

        let enable_ssl = server_table.get("enable_ssl")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(ServerConfig {
            host,
            port: port as u16,
            enable_ssl,
        })
    }

    fn parse_features(data: &HashMap<String, toml::Value>) -> HashMap<String, bool> {
        data.get("features")
            .and_then(|v| v.as_table())
            .map(|table| {
                table.iter()
                    .filter_map(|(key, value)| value.as_bool().map(|b| (key.clone(), b)))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"

            [server]
            host = "0.0.0.0"
            port = 8080
            enable_ssl = true

            [features]
            logging = true
            metrics = false
        "#;

        let temp_file = "test_config.toml";
        fs::write(temp_file, toml_content).unwrap();

        let config = Config::from_file(temp_file).unwrap();

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_ssl);
        assert_eq!(config.features.get("logging"), Some(&true));
        assert_eq!(config.features.get("metrics"), Some(&false));

        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_missing_section() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
        "#;

        let temp_file = "test_missing.toml";
        fs::write(temp_file, toml_content).unwrap();

        let result = Config::from_file(temp_file);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));

        fs::remove_file(temp_file).unwrap();
    }
}