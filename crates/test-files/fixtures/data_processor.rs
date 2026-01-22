
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {} has empty name", index));
            }
            if record.value < 0.0 {
                errors.push(format!("Record {} has negative value: {}", index, record.value));
            }
            if record.category.trim().is_empty() {
                errors.push(format!("Record {} has empty category", index));
            }
        }

        errors
    }

    pub fn get_statistics(&self) -> String {
        let avg = self.calculate_average().unwrap_or(0.0);
        let max_record = self.find_max_value();
        let validation_errors = self.validate_records();

        let max_info = match max_record {
            Some(record) => format!("Max value: {} (ID: {})", record.value, record.id),
            None => "No records found".to_string(),
        };

        format!(
            "Total records: {}\nAverage value: {:.2}\n{}\nValidation errors: {}",
            self.records.len(),
            avg,
            max_info,
            validation_errors.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let csv_data = "id,name,value,category\n1,Item1,10.5,CategoryA\n2,Item2,20.3,CategoryB\n3,Item3,15.7,CategoryA";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.1);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
        
        let stats = processor.get_statistics();
        assert!(stats.contains("Total records: 3"));
    }
}