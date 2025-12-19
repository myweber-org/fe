use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = value > 0.0 && !category.is_empty();

            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.valid).collect();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> Vec<(String, Vec<DataRecord>)> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record.clone());
            }
        }

        groups.into_iter().collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_count = self.records.iter().filter(|r| r.valid).count();
        let invalid_count = self.records.len() - valid_count;
        let avg_value = self.calculate_average().unwrap_or(0.0);
        
        Statistics {
            total: self.records.len(),
            valid: valid_count,
            invalid: invalid_count,
            average_value: avg_value,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub average_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,0.0,CategoryB").unwrap();
        writeln!(temp_file, "3,15.2,CategoryA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), (10.5 + 15.2) / 2.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.valid, 2);
        assert_eq!(stats.invalid, 1);
    }
}