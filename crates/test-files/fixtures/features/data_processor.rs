
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<f64, DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::TransformationError(
                "Invalid multiplier".to_string(),
            ));
        }

        let result = self.value * multiplier;
        if result.is_nan() || result.is_infinite() {
            Err(DataError::TransformationError(
                "Result is not finite".to_string(),
            ))
        } else {
            Ok(result)
        }
    }

    pub fn normalize(&self, min: f64, max: f64) -> Result<f64, DataError> {
        if min >= max || !min.is_finite() || !max.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid normalization range".to_string(),
            ));
        }

        let normalized = (self.value - min) / (max - min);
        if normalized.is_nan() || normalized.is_infinite() {
            Err(DataError::TransformationError(
                "Normalization failed".to_string(),
            ))
        } else {
            Ok(normalized)
        }
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, DataError>> {
    records
        .iter()
        .map(|record| record.transform(2.5))
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value > 0.0 && record.timestamp > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1234567890);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_valid() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let result = record.transform(2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 20.0);
    }

    #[test]
    fn test_normalize_valid() {
        let record = DataRecord::new(1, 75.0, 1234567890).unwrap();
        let result = record.normalize(50.0, 100.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.5);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        self.records.clear();

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let total: f64 = self.records.iter().map(|record| record.value).sum();
        Some(total / self.records.len() as f64)
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()));
        processor.add_record(DataRecord::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()));
        processor.add_record(DataRecord::new(3, "Item3".to_string(), 30.0, "CategoryA".to_string()));

        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "CategoryA"));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.calculate_average_value(), None);

        processor.add_record(DataRecord::new(1, "Item1".to_string(), 10.0, "A".to_string()));
        processor.add_record(DataRecord::new(2, "Item2".to_string(), 20.0, "B".to_string()));
        processor.add_record(DataRecord::new(3, "Item3".to_string(), 30.0, "C".to_string()));

        assert_eq!(processor.calculate_average_value(), Some(20.0));
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        let csv_content = "id,name,value,category\n1,Test1,10.5,CategoryA\n2,Test2,20.3,CategoryB\n";
        write!(temp_file, "{}", csv_content)?;

        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(processor.get_records().len(), 2);
        
        let output_path = temp_file.path().with_extension("output.csv");
        processor.save_to_csv(output_path.to_str().unwrap())?;
        
        Ok(())
    }
}