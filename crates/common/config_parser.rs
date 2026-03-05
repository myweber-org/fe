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

    fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_HOST=localhost").unwrap();
        writeln!(temp_file, "DATABASE_PORT=5432").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        parser.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(parser.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(parser.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_value");
        
        let mut parser = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "SECRET_KEY=${APP_SECRET}").unwrap();
        writeln!(temp_file, "NORMAL_VALUE=static").unwrap();

        parser.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("SECRET_KEY"), Some(&"my_secret_value".to_string()));
        assert_eq!(parser.get("NORMAL_VALUE"), Some(&"static".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut parser = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "EXISTING_KEY=value123").unwrap();

        parser.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get_or_default("EXISTING_KEY", "default"), "value123");
        assert_eq!(parser.get_or_default("MISSING_KEY", "fallback"), "fallback");
    }
}