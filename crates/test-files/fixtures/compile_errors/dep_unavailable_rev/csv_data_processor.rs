
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { headers, data })
    }
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);
        
        match column_index {
            Some(idx) => self.data.iter()
                .filter(|row| row.get(idx).map_or(false, |v| v == value))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.data.iter()
            .filter_map(|row| row.get(column_index).and_then(|v| v.parse::<f64>().ok()))
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
    
    pub fn group_by_column(&self, group_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter()
            .position(|h| h == group_column);
        let agg_idx = self.headers.iter()
            .position(|h| h == aggregate_column);
        
        let mut result = HashMap::new();
        
        if let (Some(group_idx), Some(agg_idx)) = (group_idx, agg_idx) {
            for row in &self.data {
                if let (Some(group_val), Some(agg_val)) = (row.get(group_idx), row.get(agg_idx)) {
                    if let Ok(num) = agg_val.parse::<f64>() {
                        *result.entry(group_val.clone()).or_insert(0.0) += num;
                    }
                }
            }
        }
        
        result
    }
    
    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn get_column_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000,Engineering").unwrap();
        writeln!(file, "Bob,25,45000,Marketing").unwrap();
        writeln!(file, "Charlie,35,60000,Engineering").unwrap();
        writeln!(file, "Diana,28,48000,Sales").unwrap();
        file
    }
    
    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_row_count(), 4);
        assert_eq!(processor.get_column_count(), 4);
        assert_eq!(processor.headers, vec!["name", "age", "salary", "department"]);
    }
    
    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_rows = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_rows.len(), 2);
        
        let marketing_rows = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_rows.len(), 1);
    }
    
    #[test]
    fn test_aggregate_numeric_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary", "sum");
        assert_eq!(total_salary, Some(203000.0));
        
        let avg_salary = processor.aggregate_numeric_column("salary", "avg");
        assert_eq!(avg_salary, Some(50750.0));
    }
}