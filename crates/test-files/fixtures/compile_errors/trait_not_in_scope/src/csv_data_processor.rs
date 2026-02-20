use std::error::Error;
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

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn total_records(&self) -> usize {
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,75.3,Books").unwrap();
        writeln!(temp_file, "3,ItemC,120.8,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 3);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.8,Books").unwrap();
        writeln!(temp_file, "3,ItemC,18.3,Electronics").unwrap();
        writeln!(temp_file, "4,ItemD,56.7,Books").unwrap();
        temp_file
    }

    #[test]
    fn test_load_csv_data() {
        let temp_file = create_test_csv();
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].value, 42.8);
    }

    #[test]
    fn test_filter_by_category() {
        let temp_file = create_test_csv();
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
        assert!(electronics.iter().all(|r| r.category == "Electronics"));
    }

    #[test]
    fn test_calculate_average_value() {
        let temp_file = create_test_csv();
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        let average = calculate_average_value(&records);
        assert!((average - 35.825).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value_record() {
        let temp_file = create_test_csv();
        let records = load_csv_data(temp_file.path().to_str().unwrap()).unwrap();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 4);
        assert_eq!(max_record.value, 56.7);
    }
}