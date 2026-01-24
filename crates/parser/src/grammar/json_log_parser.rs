
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, Value>,
}

pub struct LogParser {
    filters: Vec<Filter>,
    format_options: FormatOptions,
}

pub struct Filter {
    field: String,
    value: String,
    operation: FilterOperation,
}

#[derive(Clone)]
pub enum FilterOperation {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}

pub struct FormatOptions {
    show_timestamp: bool,
    show_level: bool,
    show_extra_fields: bool,
    indent_size: usize,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            format_options: FormatOptions::default(),
        }
    }

    pub fn add_filter(&mut self, field: &str, value: &str, operation: FilterOperation) {
        self.filters.push(Filter {
            field: field.to_string(),
            value: value.to_string(),
            operation,
        });
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.matches_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
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
            "timestamp" => self.compare_values(&entry.timestamp, filter),
            "level" => self.compare_values(&entry.level, filter),
            "message" => self.compare_values(&entry.message, filter),
            _ => {
                if let Some(value) = entry.extra_fields.get(&filter.field) {
                    if let Some(str_val) = value.as_str() {
                        return self.compare_values(str_val, filter);
                    }
                }
                false
            }
        }
    }

    fn compare_values(&self, actual: &str, filter: &Filter) -> bool {
        match filter.operation {
            FilterOperation::Equals => actual == filter.value,
            FilterOperation::Contains => actual.contains(&filter.value),
            FilterOperation::GreaterThan => actual > &filter.value,
            FilterOperation::LessThan => actual < &filter.value,
        }
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = String::new();

        if self.format_options.show_timestamp {
            output.push_str(&format!("[{}] ", entry.timestamp));
        }

        if self.format_options.show_level {
            output.push_str(&format!("{}: ", entry.level));
        }

        output.push_str(&entry.message);

        if self.format_options.show_extra_fields && !entry.extra_fields.is_empty() {
            output.push_str(&format!("\n{}", self.format_extra_fields(entry)));
        }

        output
    }

    fn format_extra_fields(&self, entry: &LogEntry) -> String {
        let indent = " ".repeat(self.format_options.indent_size);
        let mut fields = Vec::new();

        for (key, value) in &entry.extra_fields {
            fields.push(format!("{}{}: {}", indent, key, value));
        }

        fields.join("\n")
    }

    pub fn set_format_options(&mut self, options: FormatOptions) {
        self.format_options = options;
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            show_timestamp: true,
            show_level: true,
            show_extra_fields: false,
            indent_size: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","user":"admin"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","error_code":"500"}}"#).unwrap();

        let parser = LogParser::new();
        let entries = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Test"}}"#).unwrap();

        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR", FilterOperation::Equals);
        
        let entries = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "ERROR");
    }
}