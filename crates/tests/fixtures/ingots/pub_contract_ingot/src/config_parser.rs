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
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                value = self.substitute_env_vars(&value)?;
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }
        
        Ok(())
    }
    
    fn substitute_env_vars(&self, input: &str) -> Result<String, String> {
        let re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = input.to_string();
        
        for caps in re.captures_iter(input) {
            let var_name = &caps[1];
            match env::var(var_name) {
                Ok(val) => {
                    result = result.replace(&caps[0], &val);
                }
                Err(_) => {
                    return Err(format!("Environment variable '{}' not found", var_name));
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
        let config = r#"
            server_host = localhost
            server_port = 8080
            debug_mode = true
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_HOME", "/opt/myapp");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            data_dir = ${APP_HOME}/data
            log_dir = ${APP_HOME}/logs
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("data_dir"), Some(&"/opt/myapp/data".to_string()));
        assert_eq!(parser.get("log_dir"), Some(&"/opt/myapp/logs".to_string()));
    }
}