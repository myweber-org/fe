use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_sums
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_sums
        .into_iter()
        .map(|(category, (sum, count))| (category, sum / count as f64))
        .collect()
}

fn write_results<P: AsRef<Path>>(
    averages: &[(String, f64)],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    for (category, average) in averages {
        writer.serialize((category, average))?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv(input_path)?;
    let active_records = filter_active_records(records);
    let category_averages = calculate_category_averages(&active_records);
    write_results(&category_averages, output_path)?;

    println!("Processed {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let active_count = self.records.iter().filter(|r| r.active).count();
        let total_value: f64 = self.records.iter().map(|r| r.value).sum();
        let max_value = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        let min_value = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);

        Statistics {
            total_records: count,
            active_records: active_count,
            total_value,
            max_value,
            min_value,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub total_records: usize,
    pub active_records: usize,
    pub total_value: f64,
    pub max_value: f64,
    pub min_value: f64,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
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
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let id = parts[0].parse()?;
            let name = parts[1].to_string();
            let value = parts[2].parse()?;
            let category = parts[3].to_string();

            records.push(Record::new(id, name, value, category));
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

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "CategoryX".to_string()),
            Record::new(2, "ItemB".to_string(), 20.0, "CategoryY".to_string()),
            Record::new(3, "ItemC".to_string(), 15.0, "CategoryX".to_string()),
        ];

        let filtered = filter_by_category(&records, "CategoryX");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.0, "CatA".to_string()),
            Record::new(2, "ItemB".to_string(), 20.0, "CatA".to_string()),
        ];

        let refs: Vec<&Record> = records.iter().collect();
        let avg = calculate_average(&refs);
        assert_eq!(avg, Some(15.0));
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.0, "CatA".to_string()),
            Record::new(2, "ItemB".to_string(), 30.0, "CatB".to_string()),
            Record::new(3, "ItemC".to_string(), 20.0, "CatA".to_string()),
        ];

        let max_record = find_max_value(&records);
        assert_eq!(max_record.unwrap().id, 2);
    }
}