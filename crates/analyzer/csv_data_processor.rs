use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[&Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                value: 15.75,
                category: "Electronics".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        let avg = calculate_average(&filtered).unwrap();
        assert!((avg - 13.125).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        let max_record = find_max_value(&filtered).unwrap();
        assert_eq!(max_record.id, 3);
    }
}use std::collections::HashMap;
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>().unwrap_or(false);
                
                self.records.push(CsvRecord {
                    id,
                    category,
                    value,
                    active,
                });
                count += 1;
            }
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
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

    pub fn get_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_min_value(&self) -> Option<&CsvRecord> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();
        
        processor.add_record(CsvRecord {
            id: 1,
            category: "A".to_string(),
            value: 100.0,
            active: true,
        });
        
        processor.add_record(CsvRecord {
            id: 2,
            category: "B".to_string(),
            value: 200.0,
            active: false,
        });
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.filter_active().len(), 1);
        assert_eq!(processor.calculate_average(), 150.0);
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("A"), Some(&100.0));
        assert_eq!(aggregates.get("B"), Some(&200.0));
    }

    #[test]
    fn test_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,Electronics,299.99,true").unwrap();
        writeln!(temp_file, "2,Books,19.99,false").unwrap();
        writeln!(temp_file, "3,Electronics,599.99,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let max_record = processor.get_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 599.99);
    }
}