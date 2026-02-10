use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
struct Statistics {
    count: usize,
    total_value: f64,
    average_value: f64,
    categories: Vec<String>,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            count: 0,
            total_value: 0.0,
            average_value: 0.0,
            categories: Vec::new(),
        }
    }
}

fn process_csv_file(input_path: &str, output_path: &str) -> Result<Statistics, Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(input_file);
    
    let mut stats = Statistics::new();
    let mut records: Vec<Record> = Vec::new();
    let mut unique_categories = std::collections::HashSet::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        stats.count += 1;
        stats.total_value += record.value;
        unique_categories.insert(record.category.clone());
        
        records.push(record);
    }

    if stats.count > 0 {
        stats.average_value = stats.total_value / stats.count as f64;
    }
    
    stats.categories = unique_categories.into_iter().collect();
    stats.categories.sort();

    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(stats)
}

fn filter_records_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .collect()
}

fn calculate_category_stats(records: &[Record], category: &str) -> (f64, f64, usize) {
    let filtered: Vec<&Record> = filter_records_by_category(records, category);
    
    let count = filtered.len();
    let total: f64 = filtered.iter().map(|r| r.value).sum();
    let average = if count > 0 { total / count as f64 } else { 0.0 };
    
    (total, average, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), value: 10.5, category: "Alpha".to_string() },
            Record { id: 2, name: "Item B".to_string(), value: 20.0, category: "Beta".to_string() },
            Record { id: 3, name: "Item C".to_string(), value: 15.5, category: "Alpha".to_string() },
        ];

        let mut stats = Statistics::new();
        for record in &records {
            stats.count += 1;
            stats.total_value += record.value;
        }
        stats.average_value = stats.total_value / stats.count as f64;

        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_value, 46.0);
        assert_eq!(stats.average_value, 46.0 / 3.0);
    }

    #[test]
    fn test_category_filtering() {
        let records = vec![
            Record { id: 1, name: "Test 1".to_string(), value: 5.0, category: "X".to_string() },
            Record { id: 2, name: "Test 2".to_string(), value: 10.0, category: "Y".to_string() },
            Record { id: 3, name: "Test 3".to_string(), value: 15.0, category: "X".to_string() },
        ];

        let filtered = filter_records_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X"));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() && self.value >= 0.0
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
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(
                id,
                value,
                parts[2].to_string(),
                parts[3].to_string(),
            );

            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
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

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, -5.0, "B".to_string(), "2024-01-02".to_string());
        assert!(!invalid_record.is_valid());

        let empty_category = DataRecord::new(3, 15.0, "".to_string(), "2024-01-03".to_string());
        assert!(!empty_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);

        let record1 = DataRecord::new(1, 10.0, "A".to_string(), "2024-01-01".to_string());
        let record2 = DataRecord::new(2, 20.0, "B".to_string(), "2024-01-02".to_string());
        
        processor.records.push(record1);
        processor.records.push(record2);
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.filter_by_category("A").len(), 1);
        assert_eq!(processor.calculate_average(), Some(15.0));
    }
}