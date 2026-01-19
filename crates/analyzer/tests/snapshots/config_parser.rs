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
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*?)\s*$").unwrap();
        
        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let raw_value = caps[2].to_string();
                let processed_value = self.process_value(&raw_value)?;
                self.values.insert(key, processed_value);
            } else {
                return Err(format!("Invalid line format: {}", line));
            }
        }
        
        Ok(())
    }

    fn process_value(&self, value: &str) -> Result<String, String> {
        let env_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = value.to_string();
        
        for caps in env_regex.captures_iter(value) {
            let env_var = &caps[1];
            match env::var(env_var) {
                Ok(env_value) => {
                    result = result.replace(&caps[0], &env_value);
                }
                Err(_) => {
                    return Err(format!("Environment variable '{}' not found", env_var));
                }
            }
        }
        
        Ok(result)
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
        let config = "APP_NAME=MyApplication\nVERSION=1.0.0\nDEBUG=true";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("APP_NAME"), Some(&"MyApplication".to_string()));
        assert_eq!(parser.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut parser = ConfigParser::new();
        let config = "DATABASE_URL=${DB_HOST}:5432";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("DATABASE_URL"), Some(&"localhost:5432".to_string()));
    }
}