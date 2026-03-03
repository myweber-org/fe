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

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let mut processed_value = value.trim().to_string();

                for cap in var_pattern.captures_iter(&processed_value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                self.values.insert(key, processed_value);
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
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
        let mut parser = ConfigParser::new();
        let content = "DATABASE_URL=postgres://localhost:5432\nAPI_KEY=secret123";
        
        parser.parse_content(content).unwrap();
        assert_eq!(parser.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(parser.get("API_KEY").unwrap(), "secret123");
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PORT", "5432");
        
        let mut parser = ConfigParser::new();
        let content = "DATABASE_URL=postgres://localhost:${DB_PORT}";
        
        parser.parse_content(content).unwrap();
        assert_eq!(parser.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
    }

    #[test]
    fn test_file_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost\nPORT=8080").unwrap();
        
        let mut parser = ConfigParser::new();
        parser.parse_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("HOST").unwrap(), "localhost");
        assert_eq!(parser.get("PORT").unwrap(), "8080");
    }
}