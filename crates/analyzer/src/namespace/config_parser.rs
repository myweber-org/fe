
use std::collections::HashMap;
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
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${APP_PORT}").unwrap();
        writeln!(file, "HOST=${UNDEFINED_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "${UNDEFINED_VAR}");
    }
}