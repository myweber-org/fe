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
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s*$").unwrap();
        let env_re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                for env_cap in env_re.captures_iter(&value) {
                    let env_var = &env_cap[1];
                    if let Ok(env_value) = env::var(env_var) {
                        value = value.replace(&format!("${{{}}}", env_var), &env_value);
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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

        parser.load_from_str(config).unwrap();
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        env::set_var("DB_HOST", "postgres");

        let mut parser = ConfigParser::new();
        let config = r#"
            port = ${APP_PORT}
            database = ${DB_HOST}
            combined = server-${DB_HOST}:${APP_PORT}
        "#;

        parser.load_from_str(config).unwrap();
        assert_eq!(parser.get("port"), Some(&"3000".to_string()));
        assert_eq!(parser.get("database"), Some(&"postgres".to_string()));
        assert_eq!(parser.get("combined"), Some(&"server-postgres:3000".to_string()));
    }

    #[test]
    fn test_invalid_syntax() {
        let mut parser = ConfigParser::new();
        let config = r#"
            valid_key = value
            invalid line without equals
            another_valid = value2
        "#;

        let result = parser.load_from_str(config);
        assert!(result.is_err());
    }
}use std::collections::HashMap;
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
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
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

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}