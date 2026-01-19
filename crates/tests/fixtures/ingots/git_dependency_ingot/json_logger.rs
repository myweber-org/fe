use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct JsonLogger {
    min_level: LogLevel,
    default_fields: HashMap<String, Value>,
    output: Box<dyn Write + Send>,
}

impl JsonLogger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            default_fields: HashMap::new(),
            output: Box::new(io::stdout()),
        }
    }

    pub fn with_default_field(mut self, key: &str, value: Value) -> Self {
        self.default_fields.insert(key.to_string(), value);
        self
    }

    pub fn with_output(mut self, output: Box<dyn Write + Send>) -> Self {
        self.output = output;
        self
    }

    pub fn log(&mut self, level: LogLevel, message: &str, fields: Option<HashMap<String, Value>>) {
        if level < self.min_level {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut log_entry = json!({
            "timestamp": timestamp,
            "level": level.as_str(),
            "message": message,
        });

        if let Some(obj) = log_entry.as_object_mut() {
            for (key, value) in &self.default_fields {
                obj.insert(key.clone(), value.clone());
            }

            if let Some(additional_fields) = fields {
                for (key, value) in additional_fields {
                    obj.insert(key, value);
                }
            }
        }

        if let Ok(json_string) = serde_json::to_string(&log_entry) {
            let _ = writeln!(self.output, "{}", json_string);
        }
    }

    pub fn debug(&mut self, message: &str, fields: Option<HashMap<String, Value>>) {
        self.log(LogLevel::Debug, message, fields);
    }

    pub fn info(&mut self, message: &str, fields: Option<HashMap<String, Value>>) {
        self.log(LogLevel::Info, message, fields);
    }

    pub fn warn(&mut self, message: &str, fields: Option<HashMap<String, Value>>) {
        self.log(LogLevel::Warn, message, fields);
    }

    pub fn error(&mut self, message: &str, fields: Option<HashMap<String, Value>>) {
        self.log(LogLevel::Error, message, fields);
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };
        let other_val = match other {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        };
        Some(self_val.cmp(&other_val))
    }
}