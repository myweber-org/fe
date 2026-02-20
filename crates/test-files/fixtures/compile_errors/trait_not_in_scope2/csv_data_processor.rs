use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    active: parts[3].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_by_category(&self) -> HashMap<String, f64> {
        let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            if record.active {
                let entry = category_totals
                    .entry(record.category.clone())
                    .or_insert((0.0, 0));
                entry.0 += record.value;
                entry.1 += 1;
            }
        }
        
        category_totals
            .into_iter()
            .map(|(category, (total, count))| (category, total / count as f64))
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_active_records(&self) -> usize {
        self.records.iter().filter(|record| record.active).count()
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
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.50,true").unwrap();
        writeln!(temp_file, "2,clothing,89.99,true").unwrap();
        writeln!(temp_file, "3,electronics,150.00,false").unwrap();
        writeln!(temp_file, "4,clothing,45.75,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.get_total_records(), 4);
        assert_eq!(processor.get_active_records(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let averages = processor.calculate_average_by_category();
        assert!(averages.contains_key("electronics"));
        assert!(averages.contains_key("clothing"));
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
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
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        let mut aggregates = std::collections::HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }

        aggregates.into_iter().collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let total: f64 = self.records.iter().map(|record| record.value).sum();
        Some(total / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn find_min_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn remove_record_by_id(&mut self, id: u32) -> bool {
        let initial_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() < initial_len
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();

        let test_records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 25.0,
                active: true,
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                category: "Electronics".to_string(),
                value: 150.0,
                active: false,
            },
        ];

        for record in test_records {
            processor.add_record(record);
        }

        assert_eq!(processor.get_record_count(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);

        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 2);

        let avg = processor.calculate_average_value();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 91.66666666666667).abs() < 0.0001);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 150.0);

        let min_record = processor.find_min_value();
        assert!(min_record.is_some());
        assert_eq!(min_record.unwrap().value, 25.0);

        assert!(processor.remove_record_by_id(2));
        assert_eq!(processor.get_record_count(), 2);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }

    #[test]
    fn test_csv_io() {
        let mut processor = DataProcessor::new();

        let test_records = vec![
            Record {
                id: 1,
                name: "Test Item".to_string(),
                category: "Test".to_string(),
                value: 42.0,
                active: true,
            },
        ];

        for record in test_records {
            processor.add_record(record);
        }

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(processor.save_to_csv(path).is_ok());

        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.get_record_count(), 1);
    }
}