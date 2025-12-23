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
}