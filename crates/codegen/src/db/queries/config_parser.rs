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
        let env_var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let mut value = trimmed[equal_pos + 1..].trim().to_string();
                
                value = env_var_pattern.replace_all(&value, |caps: ®ex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| String::new())
                }).to_string();
                
                self.values.insert(key, value);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            database_url=postgres://localhost:5432
            max_connections=10
            # This is a comment
            timeout=30
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("database_url").unwrap(), "postgres://localhost:5432");
        assert_eq!(parser.get("max_connections").unwrap(), "10");
        assert_eq!(parser.get("timeout").unwrap(), "30");
    }
    
    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_PORT", "5432");
        
        let mut parser = ConfigParser::new();
        let config = r#"database_host=localhost:${DB_PORT}"#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("database_host").unwrap(), "localhost:5432");
    }
}