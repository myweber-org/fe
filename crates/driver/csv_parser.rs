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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            let _ = lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let record = self.parse_line(&line).map_err(|e| {
                format!("Line {}: {} - '{}'", line_num + 1, e, line)
            })?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str) -> Result<Vec<String>, String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' if !in_quotes => {
                    in_quotes = true;
                }
                '"' if in_quotes => {
                    if chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                }
                _ if ch == self.delimiter && !in_quotes => {
                    fields.push(current_field.clone());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        if in_quotes {
            return Err("Unclosed quotation mark".to_string());
        }

        fields.push(current_field);
        Ok(fields)
    }

    pub fn to_table(&self, records: &[Vec<String>]) -> String {
        let mut output = String::new();
        for record in records {
            output.push_str(&record.join(&self.delimiter.to_string()));
            output.push('\n');
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let parser = CsvParser::new(',', false);
        let result = parser.parse_line("a,b,c").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_quoted_fields() {
        let parser = CsvParser::new(',', false);
        let result = parser.parse_line(r#""a,b",c,"d""e""f""#).unwrap();
        assert_eq!(result, vec!["a,b", "c", "d\"e\"f"]);
    }

    #[test]
    fn test_file_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2\nvalue1,value2\nvalue3,value4").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(result, vec![
            vec!["value1", "value2"],
            vec!["value3", "value4"]
        ]);
    }
}