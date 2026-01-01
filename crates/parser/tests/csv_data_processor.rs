
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
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

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
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
            .map(|(category, (sum, count))| (category, sum / count as f64))
            .collect()
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            total_records: self.records.len(),
            average_value: self.calculate_average(),
            categories: self.get_unique_categories(),
        }
    }

    fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

pub struct DataSummary {
    total_records: usize,
    average_value: f64,
    categories: Vec<String>,
}

impl DataSummary {
    pub fn display(&self) {
        println!("Total Records: {}", self.total_records);
        println!("Average Value: {:.2}", self.average_value);
        println!("Categories: {}", self.categories.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,timestamp").unwrap();
        writeln!(file, "1,electronics,250.50,2023-01-15").unwrap();
        writeln!(file, "2,clothing,89.99,2023-01-16").unwrap();
        writeln!(file, "3,electronics,450.75,2023-01-17").unwrap();
        writeln!(file, "4,clothing,120.25,2023-01-18").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let clothing = processor.filter_by_category("clothing");
        assert_eq!(clothing.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let avg = processor.calculate_average();
        assert!((avg - 227.8725).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 2);
    }
}