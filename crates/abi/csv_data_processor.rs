
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
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
    active_count: usize,
}

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn aggregate_by_category(records: &[Record]) -> HashMap<String, AggregatedData> {
    let mut category_map: HashMap<String, (f64, usize, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0, 0));
        entry.0 += record.value;
        entry.1 += 1;
        if record.active {
            entry.2 += 1;
        }
    }

    category_map
        .into_iter()
        .map(|(category, (total, count, active_count))| {
            let aggregated = AggregatedData {
                category: category.clone(),
                total_value: total,
                average_value: total / count as f64,
                record_count: count,
                active_count,
            };
            (category, aggregated)
        })
        .collect()
}

fn process_data_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv_data(file_path)?;
    
    println!("Total records loaded: {}", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());
    
    let aggregated_data = aggregate_by_category(&records);
    
    for (category, data) in &aggregated_data {
        println!(
            "Category: {} | Total: {:.2} | Avg: {:.2} | Records: {} | Active: {}",
            category,
            data.total_value,
            data.average_value,
            data.record_count,
            data.active_count
        );
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data/sample.csv";
    
    match process_data_file(file_path) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_file(&self, file_path: &str) -> Result<bool, Box<dyn Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut record_count = 0;
        for result in rdr.records() {
            let record = result?;
            if record.is_empty() {
                return Err("Empty record found".into());
            }
            record_count += 1;
        }

        if record_count == 0 {
            return Err("No valid records found".into());
        }

        Ok(true)
    }

    pub fn transform_column(
        &self,
        file_path: &str,
        column_index: usize,
        transform_fn: fn(&str) -> String,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut transformed_values = Vec::new();
        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(column_index) {
                transformed_values.push(transform_fn(field));
            }
        }

        Ok(transformed_values)
    }

    pub fn calculate_column_stats(
        &self,
        file_path: &str,
        column_index: usize,
    ) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter as u8)
            .has_headers(self.has_headers)
            .from_reader(file);

        let mut values = Vec::new();
        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(column_index) {
                if let Ok(num) = field.parse::<f64>() {
                    values.push(num);
                }
            }
        }

        if values.is_empty() {
            return Err("No numeric values found in column".into());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transform(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "John,30,50000").unwrap();
        writeln!(file, "Jane,25,60000").unwrap();
        writeln!(file, "Bob,35,55000").unwrap();
        file
    }

    #[test]
    fn test_validate_file() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(test_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_column(
            test_file.path().to_str().unwrap(),
            0,
            uppercase_transform,
        );
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed, vec!["JOHN", "JANE", "BOB"]);
    }

    #[test]
    fn test_calculate_column_stats() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        let result = processor.calculate_column_stats(test_file.path().to_str().unwrap(), 2);
        assert!(result.is_ok());
        let (mean, variance, std_dev) = result.unwrap();
        assert!((mean - 55000.0).abs() < 0.001);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    timestamp: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        groups
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.records)?;
        Ok(())
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let average = self.calculate_average();
        let max_record = self.find_max_value();

        Statistics {
            total_records: count,
            average_value: average,
            max_value: max_record.map(|r| r.value),
            categories_count: self.group_by_category().len(),
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    total_records: usize,
    average_value: f64,
    max_value: Option<f64>,
    categories_count: usize,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Records: {}, Average: {:.2}, Max: {}, Categories: {}",
            self.total_records,
            self.average_value,
            self.max_value.unwrap_or(0.0),
            self.categories_count
        )
    }
}