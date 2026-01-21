
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV line format".into());
        }

        Ok(DataRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            timestamp: parts[4].to_string(),
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{},{}", self.id, self.name, self.category, self.value, self.timestamp)
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let record = DataRecord::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    fn save_filtered_results<P: AsRef<Path>>(
        &self,
        category: &str,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut file = File::create(output_path)?;

        writeln!(file, "id,name,category,value,timestamp")?;

        for record in filtered {
            writeln!(file, "{}", record.to_csv_line())?;
        }

        Ok(())
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

fn process_data_file() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("input_data.csv")?;

    let electronics_records = processor.filter_by_category("electronics");
    println!("Found {} electronics records", electronics_records.len());

    let aggregates = processor.aggregate_by_category();
    for (category, total, count) in aggregates {
        println!("Category: {}, Total: {:.2}, Count: {}", category, total, count);
    }

    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", mean, variance, std_dev);

    processor.save_filtered_results("electronics", "electronics_data.csv")?;

    Ok(())
}

fn main() {
    if let Err(e) = process_data_file() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.records
                .iter()
                .filter(|record| record.get(col_index).map_or(false, |v| v == value))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let col_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(col_index).and_then(|v| v.parse::<f64>().ok()))
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn group_by_column(&self, group_column: &str, agg_column: &str, operation: &str) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        
        let group_index = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return result,
        };
        
        let agg_index = match self.headers.iter().position(|h| h == agg_column) {
            Some(idx) => idx,
            None => return result,
        };

        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
        
        for record in &self.records {
            if let (Some(group_key), Some(agg_value)) = (record.get(group_index), record.get(agg_index)) {
                if let Ok(num) = agg_value.parse::<f64>() {
                    groups.entry(group_key.clone()).or_insert_with(Vec::new).push(num);
                }
            }
        }

        for (key, values) in groups {
            let aggregated = match operation {
                "sum" => values.iter().sum(),
                "avg" => values.iter().sum::<f64>() / values.len() as f64,
                "max" => values.iter().fold(f64::MIN, |a, &b| a.max(b)),
                "min" => values.iter().fold(f64::MAX, |a, &b| a.min(b)),
                _ => continue,
            };
            result.insert(key, aggregated);
        }

        result
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000.0,Engineering").unwrap();
        writeln!(file, "Bob,25,45000.0,Marketing").unwrap();
        writeln!(file, "Charlie,35,60000.0,Engineering").unwrap();
        writeln!(file, "Diana,28,48000.0,Marketing").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers().len(), 4);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let marketing_records = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_records.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let salary_sum = processor.aggregate_numeric_column("salary", "sum");
        assert_eq!(salary_sum, Some(203000.0));
        
        let salary_avg = processor.aggregate_numeric_column("salary", "avg");
        assert_eq!(salary_avg, Some(50750.0));
    }

    #[test]
    fn test_group_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let dept_salaries = processor.group_by_column("department", "salary", "sum");
        assert_eq!(dept_salaries.get("Engineering"), Some(&110000.0));
        assert_eq!(dept_salaries.get("Marketing"), Some(&93000.0));
    }
}