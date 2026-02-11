use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            category: parts[3].to_string(),
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut category_totals: HashMap<String, f64> = HashMap::new();

        for record in &self.records {
            *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
        }

        category_totals.into_iter().collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data.csv")?;

    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }

    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());

    let aggregates = processor.aggregate_by_category();
    for (category, total) in aggregates {
        println!("Category {}: total {}", category, total);
    }

    Ok(())
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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
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
                let record = CsvRecord {
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,ItemB,42.0,Books").unwrap();
        writeln!(temp_file, "3,ItemC,18.75,Electronics").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().name, "ItemB");
    }
}