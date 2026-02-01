use std::collections::HashMap;
use std::env;
use std::fs;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = self.process_value(value.trim());
                self.values.insert(key, processed_value);
            }
        }
        Ok(())
    }

    fn process_value(&self, value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
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
        let mut parser = ConfigParser::new();
        let content = "DATABASE_URL=postgres://localhost:5432\nAPI_KEY=secret123\n";
        parser.parse_content(content).unwrap();

        assert_eq!(parser.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(parser.get("API_KEY").unwrap(), "secret123");
        assert_eq!(parser.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_variable_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut parser = ConfigParser::new();
        let content = "PORT=$APP_PORT\nHOST=localhost";
        parser.parse_content(content).unwrap();

        assert_eq!(parser.get("PORT").unwrap(), "8080");
        assert_eq!(parser.get("HOST").unwrap(), "localhost");
    }

    #[test]
    fn test_file_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DEBUG=true\nLOG_LEVEL=info").unwrap();
        
        let mut parser = ConfigParser::new();
        parser.load_from_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.get("DEBUG").unwrap(), "true");
        assert_eq!(parser.get("LOG_LEVEL").unwrap(), "info");
    }

    #[test]
    fn test_get_or_default() {
        let mut parser = ConfigParser::new();
        parser.parse_content("EXISTING=value").unwrap();

        assert_eq!(parser.get_or_default("EXISTING", "default"), "value");
        assert_eq!(parser.get_or_default("MISSING", "default"), "default");
    }
}use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
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
            self.settings.insert(key, value);
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost\nport=8080\n# This is a comment").unwrap();

        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_load_invalid_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = config.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}