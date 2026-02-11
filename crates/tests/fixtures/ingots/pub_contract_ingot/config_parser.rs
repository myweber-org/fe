use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut config = Config::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid format at line {}", line_num + 1),
                ));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            config.settings.insert(key, value);
        }

        Ok(config)
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    pub fn get_parsed<T: std::str::FromStr>(&self, key: &str) -> Option<Result<T, T::Err>> {
        self.settings.get(key).map(|s| s.parse::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "timeout=30").unwrap();

        let config = Config::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.settings.len(), 3);
        assert_eq!(config.settings.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.settings.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.settings.get("timeout"), Some(&"30".to_string()));
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = Config::load_from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_with_default() {
        let mut config = Config::new();
        config.settings.insert("existing".to_string(), "value".to_string());

        assert_eq!(config.get_with_default("existing", "default"), "value");
        assert_eq!(config.get_with_default("missing", "default"), "default");
    }

    #[test]
    fn test_get_parsed() {
        let mut config = Config::new();
        config.settings.insert("number".to_string(), "42".to_string());
        config.settings.insert("invalid".to_string(), "not_a_number".to_string());

        let parsed: Option<Result<i32, _>> = config.get_parsed("number");
        assert_eq!(parsed.unwrap().unwrap(), 42);

        let invalid: Option<Result<i32, _>> = config.get_parsed("invalid");
        assert!(invalid.unwrap().is_err());

        let missing: Option<Result<i32, _>> = config.get_parsed("missing");
        assert!(missing.is_none());
    }
}