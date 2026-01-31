
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn filter_records(records: &[Record], min_value: f64) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.value >= min_value && r.active)
        .cloned()
        .collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn process_csv_file(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);
    
    let records: Vec<Record> = reader.deserialize().collect::<Result<_, _>>()?;
    
    let filtered_records = filter_records(&records, min_value);
    let aggregated_data = aggregate_by_category(&filtered_records);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    for data in aggregated_data {
        writer.serialize(data)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_records() {
        let records = vec![
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
                value: 50.0,
                active: false,
            },
        ];
        
        let filtered = filter_records(&records, 75.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
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
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
        ];
        
        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 300.0);
        assert_eq!(aggregated[0].average_value, 150.0);
    }

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "Category".to_string(),
            value: 50.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            category: "Category".to_string(),
            value: -10.0,
            active: true,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
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

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,15.2,Books").unwrap();
        writeln!(temp_file, "3,ItemC,8.7,Electronics").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(file_path).unwrap();
        
        assert_eq!(processor.get_records_count(), 3);
        assert_eq!(processor.calculate_average(), 11.466666666666667);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
    }
}