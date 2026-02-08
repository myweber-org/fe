
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn write_csv_file<P: AsRef<Path>>(path: P, records: &[Record]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

pub fn process_records(records: &mut [Record], multiplier: f64) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    
    for (index, record) in records.iter_mut().enumerate() {
        if let Err(err) = record.validate() {
            errors.push(format!("Record {}: {}", index + 1, err));
        } else {
            record.transform(multiplier);
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

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

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .records
            .iter()
            .map(|record| record.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, f64> = HashMap::new();

        for record in &self.records {
            *aggregates.entry(record.category.clone()).or_insert(0.0) += record.value;
        }

        let mut result: Vec<(String, f64)> = aggregates.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
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
            "id,name,value,category\n1,ItemA,10.5,Electronics\n2,ItemB,15.2,Books\n3,ItemC,8.7,Electronics"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path())
            .expect("Failed to load CSV");

        assert_eq!(processor.count_records(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let books = processor.filter_by_category("Books");
        assert_eq!(books.len(), 1);
    }

    #[test]
    fn test_calculations() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path())
            .expect("Failed to load CSV");

        let avg = processor.calculate_average();
        assert!((avg - 11.466).abs() < 0.001);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.name, "ItemB");
        assert_eq!(max_record.value, 15.2);
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path())
            .expect("Failed to load CSV");

        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 2);

        let books_agg = aggregates.iter().find(|(cat, _)| cat == "Books").unwrap();
        assert_eq!(books_agg.1, 15.2);

        let electronics_agg = aggregates
            .iter()
            .find(|(cat, _)| cat == "Electronics")
            .unwrap();
        assert_eq!(electronics_agg.1, 19.2);
    }
}