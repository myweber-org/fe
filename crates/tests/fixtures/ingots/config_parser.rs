use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
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
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = String::from("default");

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
                if current_section.is_empty() {
                    return Err(format!("Empty section name at line {}", line_num + 1));
                }
                config.sections.entry(current_section.clone()).or_default();
            } else if let Some(equal_pos) = trimmed.find('=') {
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

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|sec| sec.get(key))
            .map(|s| s.as_str())
    }

    pub fn get_section(&self, section: &str) -> Option<&HashMap<String, String>> {
        self.sections.get(section)
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    pub fn sections(&self) -> Vec<&str> {
        self.sections.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
# Sample config
[server]
host = 127.0.0.1
port = 8080

[database]
url = postgres://localhost/mydb
"#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("server", "host"), Some("127.0.0.1"));
        assert_eq!(config.get("server", "port"), Some("8080"));
        assert_eq!(config.get("database", "url"), Some("postgres://localhost/mydb"));
    }

    #[test]
    fn test_default_section() {
        let content = r#"key1 = value1
key2 = value2"#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("default", "key1"), Some("value1"));
        assert_eq!(config.get("default", "key2"), Some("value2"));
    }

    #[test]
    fn test_invalid_line() {
        let content = "invalid line without equals";
        assert!(Config::from_str(content).is_err());
    }
}