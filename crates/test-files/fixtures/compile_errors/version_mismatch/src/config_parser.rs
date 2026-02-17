use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::interpolate_env_vars(&raw_value)?;

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(input: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
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
                
                if var_name.is_empty() {
                    return Err("Empty environment variable name".to_string());
                }
                
                match env::var(&var_name) {
                    Ok(val) => result.push_str(&val),
                    Err(_) => return Err(format!("Environment variable '{}' not found", var_name)),
                }
            } else {
                result.push(ch);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST").unwrap(), "localhost");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/opt/app/${{APP_SECRET}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET").unwrap(), "mysecret123");
        assert_eq!(config.get("PATH").unwrap(), "/opt/app/mysecret123/data");
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "ENABLED=true").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("ENABLED", "false"), "true");
        assert_eq!(config.get_or_default("MISSING", "default_value"), "default_value");
    }
}