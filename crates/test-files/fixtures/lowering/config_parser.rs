use std::fs;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub sections: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|_| ConfigError::FileNotFound(path.as_ref().to_string_lossy().to_string()))?;

        let mut settings = HashMap::new();
        let mut sections = HashMap::new();
        let mut current_section = None;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed[1..trimmed.len()-1].trim().to_string();
                if section_name.is_empty() {
                    return Err(ConfigError::ParseError(
                        format!("Empty section name at line {}", line_num + 1)
                    ));
                }
                current_section = Some(section_name.clone());
                sections.insert(section_name, HashMap::new());
            } else if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos+1..].trim().to_string();
                
                if key.is_empty() {
                    return Err(ConfigError::ParseError(
                        format!("Empty key at line {}", line_num + 1)
                    ));
                }

                match &current_section {
                    Some(section_name) => {
                        if let Some(section) = sections.get_mut(section_name) {
                            section.insert(key, value);
                        }
                    }
                    None => {
                        settings.insert(key, value);
                    }
                }
            } else {
                return Err(ConfigError::ParseError(
                    format!("Invalid line format at line {}", line_num + 1)
                ));
            }
        }

        Ok(Config { settings, sections })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_section_value(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section).and_then(|s| s.get(key))
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), ConfigError> {
        for key in required_keys {
            if !self.settings.contains_key(*key) {
                return Err(ConfigError::ValidationError(
                    format!("Missing required setting: {}", key)
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "[database]").unwrap();
        writeln!(temp_file, "name=testdb").unwrap();
        
        let config = Config::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.get_setting("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("port"), Some(&"8080".to_string()));
        assert_eq!(config.get_section_value("database", "name"), Some(&"testdb".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        
        let config = Config::from_file(temp_file.path()).unwrap();
        
        assert!(config.validate_required(&["host"]).is_ok());
        assert!(config.validate_required(&["host", "missing"]).is_err());
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
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
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

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "APP_NAME=myapp").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "DEBUG=true").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"myapp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_PORT", "5432");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:${DB_PORT}/db").unwrap();
        writeln!(file, "MISSING_VAR=${NONEXISTENT_ENV}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(
            config.get("DATABASE_URL"),
            Some(&"postgres://localhost:5432/db".to_string())
        );
        assert_eq!(
            config.get("MISSING_VAR"),
            Some(&"${NONEXISTENT_ENV}".to_string())
        );
    }
}