
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
                
                value = self.substitute_variables(&value)?;
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }
        
        Ok(())
    }

    fn substitute_variables(&self, value: &str) -> Result<String, String> {
        let var_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        let mut result = value.to_string();
        
        for caps in var_re.captures_iter(value) {
            let var_name = &caps[1];
            if let Ok(env_value) = env::var(var_name) {
                result = result.replace(&caps[0], &env_value);
            } else if let Some(config_value) = self.values.get(var_name) {
                result = result.replace(&caps[0], config_value);
            } else {
                return Err(format!("Undefined variable: {}", var_name));
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
        let config = "app_name = MyApp\nversion = 1.0.0";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("app_name"), Some(&"MyApp".to_string()));
        assert_eq!(parser.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_variable_substitution() {
        env::set_var("HOME_DIR", "/home/user");
        
        let mut parser = ConfigParser::new();
        let config = "home = ${HOME_DIR}\nconfig_path = ${home}/config";
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("home"), Some(&"/home/user".to_string()));
        assert_eq!(parser.get("config_path"), Some(&"/home/user/config".to_string()));
    }
}