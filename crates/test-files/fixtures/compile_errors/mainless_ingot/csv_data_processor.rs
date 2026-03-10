
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn read_csv_file(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 4 {
            let id = fields[0].parse::<u32>().unwrap_or(0);
            let name = fields[1].to_string();
            let value = fields[2].parse::<f64>().unwrap_or(0.0);
            let category = fields[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                records.push(record);
            }
        }
    }

    Ok(records)
}

pub fn filter_records_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[DataRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    sum / records.len() as f64
}

pub fn process_data_pipeline(file_path: &Path, target_category: &str) -> Result<f64, Box<dyn Error>> {
    let all_records = read_csv_file(file_path)?;
    let filtered_records = filter_records_by_category(&all_records, target_category);
    
    if filtered_records.is_empty() {
        return Ok(0.0);
    }

    let average = calculate_average_value(&filtered_records);
    Ok(average)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform_value(2.5);
        assert_eq!(record.value, 25.0);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,category")?;
        writeln!(temp_file, "1,ItemA,10.5,Category1")?;
        writeln!(temp_file, "2,ItemB,20.0,Category2")?;
        writeln!(temp_file, "3,ItemC,15.5,Category1")?;

        let records = read_csv_file(temp_file.path())?;
        assert_eq!(records.len(), 3);

        let filtered = filter_records_by_category(&records, "Category1");
        assert_eq!(filtered.len(), 2);

        let average = calculate_average_value(&filtered);
        assert_eq!(average, 13.0);

        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

pub struct CsvProcessor {
    file_path: String,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn filter_records(&self, column_name: &str, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        let headers = rdr.headers()?.clone();
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        let mut filtered_records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            if record.get(column_index) == Some(filter_value) {
                filtered_records.push(record.iter().map(|s| s.to_string()).collect());
            }
        }
        
        Ok(filtered_records)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        Ok(rdr.records().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,New York").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.filter_records("city", "New York").unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Charlie", "35", "New York"]);
    }

    #[test]
    fn test_count_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();
        writeln!(temp_file, "Bob,25").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap());
        let count = processor.count_records().unwrap();
        
        assert_eq!(count, 2);
    }
}