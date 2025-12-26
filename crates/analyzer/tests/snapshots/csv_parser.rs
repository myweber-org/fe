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

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut records = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_comma_delimited() {
        let parser = CsvParser::new(',', false);
        let content = "name,age,city\nJohn,30,New York\nJane,25,London";
        
        let result = parser.parse_string(content).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "age", "city"]);
        assert_eq!(result[1], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_with_header_skip() {
        let parser = CsvParser::new(',', true);
        let content = "name,age,city\nJohn,30,New York\nJane,25,London";
        
        let result = parser.parse_string(content).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_semicolon_delimited() {
        let parser = CsvParser::new(';', false);
        let content = "name;age;city\nJohn;30;New York";
        
        let result = parser.parse_string(content).unwrap();
        assert_eq!(result[0], vec!["name", "age", "city"]);
    }

    #[test]
    fn test_parse_empty_lines() {
        let parser = CsvParser::new(',', false);
        let content = "a,b,c\n\n\nd,e,f\n";
        
        let result = parser.parse_string(content).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["d", "e", "f"]);
    }

    #[test]
    fn test_parse_file() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value")?;
        writeln!(temp_file, "1,apple,5.5")?;
        writeln!(temp_file, "2,banana,3.2")?;
        
        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path())?;
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "apple", "5.5"]);
        
        Ok(())
    }
}