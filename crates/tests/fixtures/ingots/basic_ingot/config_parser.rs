use std::collections::HashMap;
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
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=$API_KEY").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
    }
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+)$").unwrap();
        let env_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();

                for env_cap in env_re.captures_iter(&value) {
                    let env_var = &env_cap[1];
                    if let Ok(env_value) = env::var(env_var) {
                        value = value.replace(&env_cap[0], &env_value);
                    } else {
                        return Err(format!(
                            "Line {}: Environment variable '{}' not found",
                            line_num + 1,
                            env_var
                        ));
                    }
                }

                self.values.insert(key, value);
            } else {
                return Err(format!("Line {}: Invalid syntax", line_num + 1));
            }
        }

        Ok(())
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

    #[test]
    fn test_basic_parsing() {
        env::set_var("TEST_PORT", "8080");
        let mut config = Config::new();
        let content = "server_host=localhost\nserver_port=${TEST_PORT}\ndebug_mode=true";
        config.load_from_str(content).unwrap();

        assert_eq!(config.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug_mode"), Some(&"true".to_string()));
        assert_eq!(config.get("missing_key"), None);
    }

    #[test]
    fn test_env_var_missing() {
        let mut config = Config::new();
        let content = "port=${UNDEFINED_VAR}";
        let result = config.load_from_str(content);
        assert!(result.is_err());
    }
}