
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
enum CsvError {
    IoError(String),
    ParseError(usize, String),
    ValidationError(usize, String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(line, msg) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(line, msg) => write!(f, "Validation error at line {}: {}", line, msg),
        }
    }
}

impl Error for CsvError {}

struct CsvProcessor {
    delimiter: char,
    required_columns: Vec<String>,
}

impl CsvProcessor {
    fn new(delimiter: char, required_columns: Vec<String>) -> Self {
        CsvProcessor {
            delimiter,
            required_columns,
        }
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        let mut headers: Option<Vec<String>> = None;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| CsvError::IoError(e.to_string()))?;
            let line_num = line_num + 1;
            
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if line_num == 1 {
                headers = Some(fields.clone());
                self.validate_headers(&fields)?;
                continue;
            }
            
            self.validate_record(line_num, &fields)?;
            records.push(fields);
        }
        
        Ok(records)
    }
    
    fn validate_headers(&self, headers: &[String]) -> Result<(), CsvError> {
        for required in &self.required_columns {
            if !headers.contains(required) {
                return Err(CsvError::ValidationError(1, 
                    format!("Missing required column: {}", required)));
            }
        }
        Ok(())
    }
    
    fn validate_record(&self, line_num: usize, fields: &[String]) -> Result<(), CsvError> {
        if fields.iter().any(|f| f.is_empty()) {
            return Err(CsvError::ValidationError(line_num,
                "Empty field detected".to_string()));
        }
        
        for field in fields {
            if field.contains('\n') || field.contains('\r') {
                return Err(CsvError::ValidationError(line_num,
                    "Field contains newline character".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn count_records(&self, records: &[Vec<String>]) -> usize {
        records.len()
    }
    
    fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records.iter()
            .filter_map(|record| record.get(column_index))
            .cloned()
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(
        ',',
        vec!["id".to_string(), "name".to_string(), "value".to_string()]
    );
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            println!("Processed {} records", processor.count_records(&records));
            
            let names = processor.extract_column(&records, 1);
            println!("Extracted {} names", names.len());
            
            for name in names.iter().take(5) {
                println!("Name: {}", name);
            }
        }
        Err(e) => {
            eprintln!("Error processing CSV: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,Alice,100").unwrap();
        writeln!(temp_file, "2,Bob,200").unwrap();
        
        let processor = CsvProcessor::new(
            ',',
            vec!["id".to_string(), "name".to_string()]
        );
        
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][1], "Alice");
    }
    
    #[test]
    fn test_missing_required_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        
        let processor = CsvProcessor::new(
            ',',
            vec!["id".to_string(), "name".to_string()]
        );
        
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
        
        if let Err(CsvError::ValidationError(_, msg)) = result {
            assert!(msg.contains("Missing required column"));
        } else {
            panic!("Expected ValidationError");
        }
    }
}