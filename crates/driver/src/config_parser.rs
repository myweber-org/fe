use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        let mut current_section = String::from("default");

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                config.sections.entry(current_section.clone()).or_default();
                continue;
            }

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos + 1..].trim().to_string();

                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_num + 1));
                }

                config
                    .sections
                    .entry(current_section.clone())
                    .or_default()
                    .insert(key, value);
            } else {
                return Err(format!("Invalid line format at line {}", line_num + 1));
            }
        }

        Ok(config)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = Config::new();
        assert!(config.sections.is_empty());
    }

    #[test]
    fn test_parse_basic() {
        let content = "[database]\nhost=localhost\nport=5432\n\n[server]\naddress=0.0.0.0\nport=8080";
        let temp_file = "test_config.ini";
        fs::write(temp_file, content).unwrap();

        let config = Config::from_file(temp_file).unwrap();
        fs::remove_file(temp_file).unwrap();

        assert_eq!(config.get("database", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("database", "port"), Some(&"5432".to_string()));
        assert_eq!(config.get("server", "address"), Some(&"0.0.0.0".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_set_value() {
        let mut config = Config::new();
        config.set("app", "debug", "true");
        assert_eq!(config.get("app", "debug"), Some(&"true".to_string()));
    }
}