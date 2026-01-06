use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            headers: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_line) = lines.next() {
            let header_line = header_line?;
            self.headers = header_line.split(',').map(|s| s.trim().to_string()).collect();
        }

        for line_result in lines {
            let line = line_result?;
            let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if values.len() == self.headers.len() {
                let record = CsvRecord {
                    columns: self.headers.clone(),
                    values,
                };
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<CsvRecord> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record.values[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records.iter()
            .filter_map(|record| record.values[column_index].parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn group_by_column(&self, column_name: &str) -> HashMap<String, Vec<CsvRecord>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        let mut groups: HashMap<String, Vec<CsvRecord>> = HashMap::new();

        if let Some(idx) = column_index {
            for record in &self.records {
                let key = record.values[idx].clone();
                groups.entry(key).or_default().push(record.clone());
            }
        }

        groups
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
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
        writeln!(temp_file, "1,item_a,10.5").unwrap();
        writeln!(temp_file, "2,item_b,20.3").unwrap();
        writeln!(temp_file, "3,item_a,15.7").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.get_headers(), &vec!["id".to_string(), "name".to_string(), "value".to_string()]);

        let filtered = processor.filter_by_column("name", |name| name == "item_a");
        assert_eq!(filtered.len(), 2);

        let sum = processor.aggregate_numeric_column("value", "sum");
        assert_eq!(sum, Some(46.5));

        let groups = processor.group_by_column("name");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("item_a").unwrap().len(), 2);
    }
}