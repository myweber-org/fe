use serde_json::{json, Value};
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct JsonLogger {
    min_level: LogLevel,
    output: Box<dyn Write + Send>,
    include_timestamp: bool,
    service_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl JsonLogger {
    pub fn new(service_name: &str) -> Self {
        Self {
            min_level: LogLevel::Info,
            output: Box::new(io::stdout()),
            include_timestamp: true,
            service_name: service_name.to_string(),
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn with_output(mut self, output: Box<dyn Write + Send>) -> Self {
        self.output = output;
        self
    }

    pub fn log(&mut self, level: LogLevel, message: &str, fields: Option<Value>) {
        if level > self.min_level {
            return;
        }

        let mut log_entry = json!({
            "level": format!("{:?}", level).to_uppercase(),
            "message": message,
            "service": self.service_name,
        });

        if self.include_timestamp {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            log_entry["timestamp"] = json!(timestamp);
        }

        if let Some(fields) = fields {
            if let Some(obj) = log_entry.as_object_mut() {
                if let Some(fields_obj) = fields.as_object() {
                    for (key, value) in fields_obj {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        if let Ok(json_string) = serde_json::to_string(&log_entry) {
            let _ = writeln!(self.output, "{}", json_string);
        }
    }

    pub fn error(&mut self, message: &str, fields: Option<Value>) {
        self.log(LogLevel::Error, message, fields);
    }

    pub fn warn(&mut self, message: &str, fields: Option<Value>) {
        self.log(LogLevel::Warn, message, fields);
    }

    pub fn info(&mut self, message: &str, fields: Option<Value>) {
        self.log(LogLevel::Info, message, fields);
    }

    pub fn debug(&mut self, message: &str, fields: Option<Value>) {
        self.log(LogLevel::Debug, message, fields);
    }

    pub fn trace(&mut self, message: &str, fields: Option<Value>) {
        self.log(LogLevel::Trace, message, fields);
    }
}