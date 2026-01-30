use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn without_header(mut self) -> Self {
        self.has_header = false;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                if idx == 0 && self.has_header {
                    None
                } else {
                    let fields: Vec<String> = line
                        .split(self.delimiter)
                        .map(|s| s.trim().to_string())
                        .collect();
                    if fields.is_empty() {
                        None
                    } else {
                        Some(fields)
                    }
                }
            })
            .collect()
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_with_header() {
        let parser = CsvParser::new();
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_string_without_header() {
        let parser = CsvParser::new().without_header();
        let csv_data = "John,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().with_delimiter(';');
        let csv_data = "name;age;city\nJohn;30;New York";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_fields() {
        let parser = CsvParser::new();
        let csv_data = "a,b,c\n1,,3\n4,5,";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result[0], vec!["1", "", "3"]);
        assert_eq!(result[1], vec!["4", "5", ""]);
    }
}