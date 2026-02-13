use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
}

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid column count at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            format!("Failed to parse ID at line {}: {}", line_num + 1, e)
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_num + 1).into());
        }

        let value = parts[2].parse::<f64>().map_err(|e| {
            format!("Failed to parse value at line {}: {}", line_num + 1, e)
        })?;

        records.push(CsvRecord { id, name, value });
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ProductA,25.5").unwrap();
        writeln!(temp_file, "2,ProductB,30.0").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,ProductC,42.75").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "ProductA");
        assert_eq!(records[1].value, 30.0);
        assert_eq!(records[2].id, 3);
    }

    #[test]
    fn test_invalid_csv_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,ProductA").unwrap();

        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0 },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0 },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0 },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}