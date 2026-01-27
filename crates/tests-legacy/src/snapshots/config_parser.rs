use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config = Self::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }
        
        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn merge(&mut self, other: &Config) {
        for (key, value) in &other.values {
            self.values.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
    }

    #[test]
    fn test_env_override() {
        env::set_var("TEST_KEY", "env_value");
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");
        
        assert_eq!(config.get("TEST_KEY"), Some("env_value".to_string()));
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING_KEY", "default_value"), "default_value");
    }
}use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct ConfigSection {
    pub values: HashMap<String, String>,
    pub subsections: HashMap<String, ConfigSection>,
}

impl ConfigSection {
    pub fn new() -> Self {
        ConfigSection {
            values: HashMap::new(),
            subsections: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_subsection(&self, name: &str) -> Option<&ConfigSection> {
        self.subsections.get(name)
    }

    pub fn resolve_env_vars(&mut self) {
        let mut resolved_values = HashMap::new();
        
        for (key, value) in &self.values {
            let resolved = Self::replace_env_vars(value);
            resolved_values.insert(key.clone(), resolved);
        }
        
        self.values = resolved_values;
        
        for subsection in self.subsections.values_mut() {
            subsection.resolve_env_vars();
        }
    }

    fn replace_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

pub struct ConfigParser;

impl ConfigParser {
    pub fn parse(content: &str) -> Result<ConfigSection, String> {
        let mut root = ConfigSection::new();
        let mut current_path: Vec<String> = Vec::new();
        let mut current_section = &mut root;
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = &trimmed[1..trimmed.len() - 1];
                let parts: Vec<&str> = section_name.split('.').collect();
                
                current_path = parts.iter().map(|&s| s.to_string()).collect();
                current_section = Self::get_or_create_section(&mut root, &current_path);
            } else if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos + 1..].trim().to_string();
                
                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_num + 1));
                }
                
                current_section.values.insert(key, value);
            } else {
                return Err(format!("Invalid line format at line {}", line_num + 1));
            }
        }
        
        Ok(root)
    }
    
    fn get_or_create_section<'a>(
        root: &'a mut ConfigSection,
        path: &[String]
    ) -> &'a mut ConfigSection {
        let mut current = root;
        
        for section_name in path {
            current = current.subsections
                .entry(section_name.clone())
                .or_insert_with(ConfigSection::new);
        }
        
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let config = r#"
[server]
host = localhost
port = 8080

[database]
url = postgresql://localhost/mydb
pool_size = 10
"#;
        
        let parsed = ConfigParser::parse(config).unwrap();
        
        assert_eq!(parsed.get_subsection("server").unwrap().get("host"), Some(&"localhost".to_string()));
        assert_eq!(parsed.get_subsection("server").unwrap().get("port"), Some(&"8080".to_string()));
        assert_eq!(parsed.get_subsection("database").unwrap().get("url"), Some(&"postgresql://localhost/mydb".to_string()));
    }

    #[test]
    fn test_nested_sections() {
        let config = r#"
[server.api.v1]
endpoint = /api/v1
timeout = 30
"#;
        
        let parsed = ConfigParser::parse(config).unwrap();
        let api_section = parsed.get_subsection("server")
            .unwrap()
            .get_subsection("api")
            .unwrap()
            .get_subsection("v1")
            .unwrap();
        
        assert_eq!(api_section.get("endpoint"), Some(&"/api/v1".to_string()));
        assert_eq!(api_section.get("timeout"), Some(&"30".to_string()));
    }
}