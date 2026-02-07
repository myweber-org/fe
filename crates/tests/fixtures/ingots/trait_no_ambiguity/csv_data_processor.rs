
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        DataRecord {
            id,
            category,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.value > 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut wtr = csv::Writer::from_writer(writer);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.is_valid()).collect();
        
        if valid_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        sum / valid_records.len() as f64
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn remove_invalid_records(&mut self) {
        self.records.retain(|r| r.is_valid());
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, true);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "A".to_string(), 10.0, true));
        processor.add_record(DataRecord::new(2, "B".to_string(), 20.0, true));
        processor.add_record(DataRecord::new(3, "C".to_string(), 30.0, true));

        assert_eq!(processor.calculate_average(), 20.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "X".to_string(), 10.0, true));
        processor.add_record(DataRecord::new(2, "Y".to_string(), 20.0, false));
        processor.add_record(DataRecord::new(3, "X".to_string(), 30.0, true));

        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X" && r.active));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }
        
        Ok(CsvProcessor { headers, records })
    }
    
    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);
        
        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
    
    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let processor = CsvProcessor::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 3);
        
        let filtered = processor.filter_by_column("age", |age| age.parse::<i32>().unwrap() >= 30);
        assert_eq!(filtered.len(), 2);
        
        let avg_salary = processor.aggregate_numeric_column("salary");
        assert!(avg_salary.is_some());
        assert!((avg_salary.unwrap() - 51666.666).abs() < 0.001);
    }
}