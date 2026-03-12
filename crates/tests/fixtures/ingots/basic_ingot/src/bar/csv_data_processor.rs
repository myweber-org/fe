use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers: Vec<String> = if let Some(first_line) = lines.next() {
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

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: AggregationOp) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        match operation {
            AggregationOp::Sum => Some(values.iter().sum()),
            AggregationOp::Average => Some(values.iter().sum::<f64>() / values.len() as f64),
            AggregationOp::Max => values.iter().copied().reduce(f64::max),
            AggregationOp::Min => values.iter().copied().reduce(f64::min),
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<ColumnStats> {
        let values = self.aggregate_numeric_column(column_name, AggregationOp::Average)?;
        let count = self.records.len();
        let sum = self.aggregate_numeric_column(column_name, AggregationOp::Sum)?;
        let max = self.aggregate_numeric_column(column_name, AggregationOp::Max)?;
        let min = self.aggregate_numeric_column(column_name, AggregationOp::Min)?;

        Some(ColumnStats {
            column_name: column_name.to_string(),
            count,
            sum,
            average: values,
            max,
            min,
        })
    }
}

pub enum AggregationOp {
    Sum,
    Average,
    Max,
    Min,
}

pub struct ColumnStats {
    pub column_name: String,
    pub count: usize,
    pub sum: f64,
    pub average: f64,
    pub max: f64,
    pub min: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Column: {}\nCount: {}\nSum: {:.2}\nAverage: {:.2}\nMax: {:.2}\nMin: {:.2}",
            self.column_name, self.count, self.sum, self.average, self.max, self.min
        )
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        lines.next();

        for line_result in lines {
            let line = line_result?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let name = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let category = parts[3].to_string();

                self.records.push(Record {
                    id,
                    name,
                    value,
                    category,
                });
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn clear_records(&mut self) {
        self.records.clear();
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.get_records_count(), 0);
        assert_eq!(processor.calculate_average(), 0.0);
    }

    #[test]
    fn test_add_record() {
        let mut processor = CsvProcessor::new();
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        processor.add_record(record);
        assert_eq!(processor.get_records_count(), 1);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = CsvProcessor::new();
        
        processor.add_record(Record {
            id: 1,
            name: "Item1".to_string(),
            value: 10.0,
            category: "CategoryA".to_string(),
        });

        processor.add_record(Record {
            id: 2,
            name: "Item2".to_string(),
            value: 20.0,
            category: "CategoryB".to_string(),
        });

        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}