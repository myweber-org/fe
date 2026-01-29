use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record {
                id,
                name,
                value,
                category,
            });
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

pub fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|r| r.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    let mut category_totals = std::collections::HashMap::new();

    for record in records {
        let entry = category_totals.entry(record.category.clone()).or_insert(0.0);
        *entry += record.value;
    }

    category_totals
        .into_iter()
        .map(|(category, total)| (category, total))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<Record> {
        vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                value: 100.0,
                category: "Electronics".to_string(),
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                value: 200.0,
                category: "Electronics".to_string(),
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                value: 150.0,
                category: "Books".to_string(),
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
    fn test_calculate_average_value() {
        let records = create_test_records();
        let average = calculate_average_value(&records);
        assert!((average - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = create_test_records();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert!((max_record.value - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = create_test_records();
        let aggregates = aggregate_by_category(&records);
        
        let electronics_total: f64 = aggregates
            .iter()
            .find(|(cat, _)| cat == "Electronics")
            .map(|(_, total)| *total)
            .unwrap_or(0.0);
        
        assert!((electronics_total - 300.0).abs() < 0.001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn from_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(CsvRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
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
            let record = CsvRecord::from_line(&line)?;
            self.records.push(record);
        }

        Ok(())
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

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_category_summary(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = category_map
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        category_map
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    pub fn count_records(&self) -> usize {
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

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,category,value,active\n1,ItemA,Electronics,100.5,true\n2,ItemB,Books,25.0,false\n3,ItemC,Electronics,75.0,true"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(test_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_by_category() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let books = processor.filter_by_category("Books");
        assert_eq!(books.len(), 1);
    }

    #[test]
    fn test_calculate_total() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();

        let total = processor.calculate_total_value();
        assert_eq!(total, 200.5);
    }

    #[test]
    fn test_find_max_value() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();

        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 1);
        assert_eq!(max_record.value, 100.5);
    }
}