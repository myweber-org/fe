
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub valid_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(ValidationError::InvalidId);
        }
        
        self.records.insert(record.id, record.clone());
        self.update_statistics(&record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            record.transform(multiplier);
        }
        self.recalculate_statistics();
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        let removed = self.records.remove(&id);
        if removed.is_some() {
            self.recalculate_statistics();
        }
        removed
    }

    pub fn filter_by_min_value(&self, min_value: f64) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.value >= min_value)
            .collect()
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        self.statistics.total_records += 1;
        self.statistics.valid_records += 1;
        self.statistics.total_value += record.value;
        self.statistics.average_value = self.statistics.total_value / self.statistics.valid_records as f64;
    }

    fn recalculate_statistics(&mut self) {
        self.statistics = ProcessingStats::default();
        for record in self.records.values() {
            self.update_statistics(record);
        }
    }
}

pub fn create_sample_data() -> Vec<DataRecord> {
    vec![
        DataRecord::new(1, "alpha".to_string(), 10.5, vec!["tag1".to_string(), "tag2".to_string()]),
        DataRecord::new(2, "beta".to_string(), 20.0, vec!["tag3".to_string()]),
        DataRecord::new(3, "gamma".to_string(), 15.75, vec!["tag1".to_string(), "tag4".to_string()]),
    ]
}