
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if values.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Dataset contains invalid numeric values".to_string());
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        self.data.get(key).map(|values| {
            let stats = self.calculate_statistics(key).unwrap();
            values.iter()
                .map(|&x| (x - stats.mean) / stats.std_dev)
                .collect()
        })
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Count: {}, Sum: {:.2}, Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}",
            self.count, self.sum, self.mean, self.variance, self.std_dev
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(processor.get_keys(), vec!["test"]);
    }

    #[test]
    fn test_add_invalid_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("test".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
    }

    #[test]
    fn test_normalize_data() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0]).unwrap();
        
        let normalized = processor.normalize_data("test").unwrap();
        assert_eq!(normalized.len(), 3);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_statistics();
        self.normalize_values()?;
        self.validate_processed_data()?;

        Ok(())
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn get_all_statistics(&self) -> &HashMap<String, CategoryStatistics> {
        &self.category_stats
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(&mut DataRecord) -> Result<(), ProcessingError>,
    {
        for record in &mut self.records {
            transform_fn(record)?;
        }
        self.recalculate_statistics();
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError("Record name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Record value cannot be negative".to_string()));
        }
        
        if record.category.is_empty() {
            return Err(ProcessingError::ValidationError("Category cannot be empty".to_string()));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });

        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    fn calculate_statistics(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Ok(());
        }

        let max_value = self.records
            .iter()
            .map(|r| r.value)
            .fold(f64::MIN, |a, b| a.max(b));

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize: maximum value is zero or negative".to_string()
            ));
        }

        for record in &mut self.records {
            record.value = record.value / max_value;
        }

        self.recalculate_statistics();
        Ok(())
    }

    fn validate_processed_data(&self) -> Result<(), ProcessingError> {
        for stats in self.category_stats.values() {
            if stats.average_value.is_nan() || stats.average_value.is_infinite() {
                return Err(ProcessingError::ValidationError(
                    "Invalid average value in statistics".to_string()
                ));
            }
        }
        Ok(())
    }

    fn recalculate_statistics(&mut self) {
        self.calculate_statistics();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_statistics() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 75.0,
                category: "CategoryB".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        processor.process_records().unwrap();

        let stats_a = processor.get_category_statistics("CategoryA").unwrap();
        assert_eq!(stats_a.total_value, 150.0);
        assert_eq!(stats_a.record_count, 2);
        assert_eq!(stats_a.average_value, 75.0);

        let stats_b = processor.get_category_statistics("CategoryB").unwrap();
        assert_eq!(stats_b.total_value, 75.0);
        assert_eq!(stats_b.record_count, 1);
        assert_eq!(stats_b.average_value, 75.0);
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 25.0,
                category: "Test".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 50.0,
                category: "Test".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 75.0,
                category: "Test".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        processor.process_records().unwrap();
        
        let normalized_values: Vec<f64> = processor.records
            .iter()
            .map(|r| r.value)
            .collect();

        assert!(normalized_values.iter().all(|&v| v >= 0.0 && v <= 1.0));
        assert!(normalized_values.contains(&1.0));
    }
}