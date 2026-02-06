use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
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
        let mut lines = reader.lines();

        // Skip header
        lines.next();

        for line in lines {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>()?;

                self.records.push(Record {
                    id,
                    category,
                    value,
                    active,
                });
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }
        
        aggregates
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn get_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        // Create test CSV
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,100.5,true").unwrap();
        writeln!(temp_file, "2,clothing,50.25,false").unwrap();
        writeln!(temp_file, "3,electronics,75.0,true").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        // Test loading
        assert!(processor.load_from_csv(file_path).is_ok());
        assert_eq!(processor.count_records(), 3);
        
        // Test filtering
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let active = processor.filter_active();
        assert_eq!(active.len(), 2);
        
        // Test aggregation
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("electronics"), Some(&175.5));
        
        // Test calculations
        let avg = processor.calculate_average();
        assert!((avg - 75.25).abs() < 0.001);
        
        let max_record = processor.get_max_value().unwrap();
        assert_eq!(max_record.value, 100.5);
        
        let min_record = processor.get_min_value().unwrap();
        assert_eq!(min_record.value, 50.25);
    }
}