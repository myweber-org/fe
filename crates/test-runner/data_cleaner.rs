use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr(&mut self) -> (f64, f64, f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("iqr".to_string(), iqr);
        self.thresholds.insert("q1".to_string(), q1);
        self.thresholds.insert("q3".to_string(), q3);

        (q1, q3, iqr, lower_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower_bound = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper_bound = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= *lower_bound && value <= *upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        self.thresholds.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_iqr_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut cleaner = DataCleaner::new(data);
        
        let (q1, q3, iqr, _) = cleaner.calculate_iqr();
        
        assert!(q1 > 0.0);
        assert!(q3 > q1);
        assert!(iqr > 0.0);
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> usize {
        let mut unique_set = HashSet::new();
        let mut deduped_records = Vec::new();
        
        for record in self.records.drain(..) {
            if unique_set.insert(record.clone()) {
                deduped_records.push(record);
            }
        }
        
        let removed_count = self.records.len() - deduped_records.len();
        self.records = deduped_records;
        removed_count
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty() 
                && record.len() <= 1000
                && record.chars().all(|c| c.is_ascii() || c.is_alphanumeric())
            })
            .collect()
    }

    pub fn get_valid_records(&self) -> Vec<&String> {
        let validation_results = self.validate_records();
        self.records
            .iter()
            .zip(validation_results)
            .filter(|(_, is_valid)| *is_valid)
            .map(|(record, _)| record)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid123".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("a".repeat(1001));
        
        let valid_records = cleaner.get_valid_records();
        assert_eq!(valid_records.len(), 1);
        assert_eq!(*valid_records[0], "valid123");
    }
}use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    for result in csv_reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                if field.trim().is_empty() || field.eq_ignore_ascii_case("null") {
                    String::from("")
                } else {
                    field.to_string()
                }
            })
            .collect();

        csv_writer.write_record(&cleaned_record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "Alice,25,New York").unwrap();
        writeln!(input_file, "Bob,null,London").unwrap();
        writeln!(input_file, "Charlie,30,").unwrap();
        writeln!(input_file, "David,40,NULL").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();

        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Alice,25,New York"));
        assert!(content.contains("Bob,,London"));
        assert!(content.contains("Charlie,30,"));
        assert!(content.contains("David,40,"));
    }
}