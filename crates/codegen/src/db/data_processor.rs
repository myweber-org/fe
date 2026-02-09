
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) {
        self.data.insert(name, values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            match self.data.get(&rule.field_name) {
                Some(values) => {
                    if rule.required && values.is_empty() {
                        errors.push(format!("Field '{}' is required but empty", rule.field_name));
                    }

                    for (index, &value) in values.iter().enumerate() {
                        if value < rule.min_value || value > rule.max_value {
                            errors.push(format!(
                                "Value {} at index {} in field '{}' is out of range [{}, {}]",
                                value, index, rule.field_name, rule.min_value, rule.max_value
                            ));
                        }
                    }
                }
                None if rule.required => {
                    errors.push(format!("Required field '{}' not found", rule.field_name));
                }
                None => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if let Some(&max) = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                if max != 0.0 {
                    for value in values.iter_mut() {
                        *value /= max;
                    }
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> HashMap<String, Statistics> {
        let mut stats = HashMap::new();

        for (name, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;

            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;

            stats.insert(name.clone(), Statistics {
                mean,
                variance,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: values.len(),
            });
        }

        stats
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
            min_value,
            max_value,
            required,
        }
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let active = parts[3].parse::<bool>()?;

            self.records.push(Record {
                id,
                name,
                value,
                active,
            });
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,Test1,10.5,true").unwrap();
        writeln!(file, "2,Test2,20.0,false").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.record_count(), 2);
    }

    #[test]
    fn test_filter_active() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "A".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "B".to_string(),
            value: 20.0,
            active: false,
        });

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 1,
            name: "A".to_string(),
            value: 10.0,
            active: true,
        });
        processor.records.push(Record {
            id: 2,
            name: "B".to_string(),
            value: 20.0,
            active: true,
        });

        assert_eq!(processor.calculate_average(), Some(15.0));
    }

    #[test]
    fn test_find_by_id() {
        let mut processor = DataProcessor::new();
        processor.records.push(Record {
            id: 42,
            name: "Test".to_string(),
            value: 100.0,
            active: true,
        });

        let found = processor.find_by_id(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 42);
        
        let not_found = processor.find_by_id(99);
        assert!(not_found.is_none());
    }
}