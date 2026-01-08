
use std::collections::HashMap;
use std::env;
use std::fs;
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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let var_regex = Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let mut value = trimmed[equal_pos + 1..].trim().to_string();

                for cap in var_regex.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }

                self.values.insert(key, value);
            }
        }

        Ok(())
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

    pub fn get_all(&self) -> &HashMap<String, String> {
        &self.values
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
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "APP_NAME=MyApp").unwrap();

        let mut parser = ConfigParser::new();
        parser.load_from_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(parser.get("APP_NAME"), Some(&"MyApp".to_string()));
        assert_eq!(parser.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${DB_PASSWORD}").unwrap();
        writeln!(file, "CONNECTION=postgres://user:${DB_PASSWORD}@localhost").unwrap();

        let mut parser = ConfigParser::new();
        parser.load_from_file(file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.get("DB_PASS"), Some(&"secret123".to_string()));
        assert_eq!(
            parser.get("CONNECTION"),
            Some(&"postgres://user:secret123@localhost".to_string())
        );
    }
}