
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let record = DataRecord::new(id, value, category);

            if record.is_valid() {
                self.total_value += record.get_value();
                self.valid_count += 1;
            }

            self.records.push(record);
        }

        Ok(())
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid_records(&self) -> usize {
        self.valid_count
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,-5.0,type_b").unwrap();
        writeln!(temp_file, "3,20.0,type_a").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid_records(), 2);
        
        let avg = processor.get_average_value().unwrap();
        assert!((avg - 15.25).abs() < 0.001);
        
        let type_a_records = processor.filter_by_category("type_a");
        assert_eq!(type_a_records.len(), 2);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);

        Statistics {
            count,
            average: avg,
            max_value: max,
            min_value: min,
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub max_value: f64,
    pub min_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.3,Beta\n3,ItemC,8.7,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.filter_by_category("Alpha").len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.average - 11.5).abs() < 0.01);
        assert_eq!(stats.max_value, 15.3);
        assert_eq!(stats.min_value, 8.7);
    }
}