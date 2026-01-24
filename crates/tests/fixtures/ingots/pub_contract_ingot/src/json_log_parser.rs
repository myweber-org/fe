use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filters: Vec<Filter>,
    format_options: FormatOptions,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub value: Value,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone)]
pub struct FormatOptions {
    pub show_timestamp: bool,
    pub show_level: bool,
    pub show_fields: bool,
    pub indent: usize,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            format_options: FormatOptions {
                show_timestamp: true,
                show_level: true,
                show_fields: false,
                indent: 2,
            },
        }
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                if self.matches_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO")
            .to_string();
            
        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn matches_filters(&self, entry: &LogEntry) -> bool {
        for filter in &self.filters {
            if !self.matches_filter(entry, filter) {
                return false;
            }
        }
        true
    }

    fn matches_filter(&self, entry: &LogEntry, filter: &Filter) -> bool {
        match filter.field.as_str() {
            "level" => self.compare_values(&Value::String(entry.level.clone()), &filter.value, &filter.operator),
            "message" => self.compare_values(&Value::String(entry.message.clone()), &filter.value, &filter.operator),
            _ => {
                if let Some(field_value) = entry.fields.get(&filter.field) {
                    self.compare_values(field_value, &filter.value, &filter.operator)
                } else {
                    false
                }
            }
        }
    }

    fn compare_values(&self, a: &Value, b: &Value, operator: &FilterOperator) -> bool {
        match operator {
            FilterOperator::Equals => a == b,
            FilterOperator::Contains => {
                if let (Some(a_str), Some(b_str)) = (a.as_str(), b.as_str()) {
                    a_str.contains(b_str)
                } else {
                    false
                }
            }
            FilterOperator::GreaterThan => {
                if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
                    a_num > b_num
                } else {
                    false
                }
            }
            FilterOperator::LessThan => {
                if let (Some(a_num), Some(b_num)) = (a.as_f64(), b.as_f64()) {
                    a_num < b_num
                } else {
                    false
                }
            }
        }
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();
        
        if self.format_options.show_timestamp && !entry.timestamp.is_empty() {
            parts.push(format!("[{}]", entry.timestamp));
        }
        
        if self.format_options.show_level {
            parts.push(format!("{}:", entry.level));
        }
        
        parts.push(entry.message.clone());
        
        let mut result = parts.join(" ");
        
        if self.format_options.show_fields && !entry.fields.is_empty() {
            let fields_str = serde_json::to_string_pretty(&entry.fields)
                .unwrap_or_else(|_| "{}".to_string());
            result.push_str(&format!("\n{}", fields_str));
        }
        
        result
    }

    pub fn set_format_options(&mut self, options: FormatOptions) {
        self.format_options = options;
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}