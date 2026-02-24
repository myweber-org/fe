use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_env_or_config(map, "DATABASE_URL", "database_url")?;
        let api_key = Self::get_env_or_config(map, "API_KEY", "api_key")?;
        let debug_mode = Self::parse_bool(map.get("debug_mode").map(|s| s.as_str()))?;
        let port = Self::parse_port(map.get("port").map(|s| s.as_str()))?;

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }

    fn get_env_or_config(
        map: &HashMap<String, String>,
        env_var: &str,
        config_key: &str,
    ) -> Result<String, String> {
        if let Ok(env_value) = env::var(env_var) {
            return Ok(env_value);
        }
        map.get(config_key)
            .cloned()
            .ok_or_else(|| format!("Missing configuration: {}", config_key))
    }

    fn parse_bool(value: Option<&str>) -> Result<bool, String> {
        match value {
            Some("true") | Some("1") => Ok(true),
            Some("false") | Some("0") => Ok(false),
            Some(v) => Err(format!("Invalid boolean value: {}", v)),
            None => Ok(false),
        }
    }

    fn parse_port(value: Option<&str>) -> Result<u16, String> {
        match value {
            Some(v) => v
                .parse()
                .map_err(|e| format!("Invalid port number: {}", e)),
            None => Ok(8080),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "database_url=postgres://localhost/test\napi_key=secret123\ndebug_mode=true\nport=3000"
        )
        .unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "secret123");
        assert!(config.debug_mode);
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        let map = HashMap::from([
            ("database_url".to_string(), "postgres://localhost/test".to_string()),
            ("api_key".to_string(), "secret123".to_string()),
        ]);

        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        env::remove_var("DATABASE_URL");
    }
}