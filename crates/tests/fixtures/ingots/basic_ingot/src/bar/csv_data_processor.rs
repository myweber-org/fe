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
}