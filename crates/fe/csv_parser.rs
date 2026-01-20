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
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
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

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            records.push(fields);
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
                } else if line.trim().is_empty() {
                    None
                } else {
                    Some(
                        line.split(self.delimiter)
                            .map(|s| s.trim().to_string())
                            .collect(),
                    )
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_with_comma_delimiter() {
        let parser = CsvParser::new(',', true);
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_parse_with_semicolon_delimiter() {
        let parser = CsvParser::new(';', false);
        let csv_data = "Apple;1.99;Fruit\nBread;2.50;Bakery";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Apple", "1.99", "Fruit"]);
    }

    #[test]
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,status").unwrap();
        writeln!(temp_file, "1,100,active").unwrap();
        writeln!(temp_file, "2,200,inactive").unwrap();
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "100", "active"]);
    }

    #[test]
    fn test_empty_lines_skipped() {
        let parser = CsvParser::new(',', false);
        let csv_data = "a,b,c\n\n\nd,e,f\n\n";
        let result = parser.parse_string(csv_data);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["d", "e", "f"]);
    }
}