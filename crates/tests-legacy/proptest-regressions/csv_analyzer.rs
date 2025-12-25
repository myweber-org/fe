use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn analyze_csv(file_path: &str, column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    let mut values = Vec::new();
    let mut line_count = 0;
    
    for line in reader.lines() {
        let line = line?;
        line_count += 1;
        
        if line_count == 1 {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').collect();
        if column_index < parts.len() {
            if let Ok(value) = parts[column_index].parse::<f64>() {
                values.push(value);
            }
        }
    }
    
    if values.is_empty() {
        return Err("No valid numeric data found".into());
    }
    
    let mean = calculate_mean(&values);
    let median = calculate_median(&mut values);
    
    Ok((mean, median))
}

fn calculate_mean(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

fn calculate_median(values: &mut Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();
        
        let result = analyze_csv(temp_file.path().to_str().unwrap(), 2);
        assert!(result.is_ok());
        
        let (mean, median) = result.unwrap();
        assert_eq!(mean, 50000.0);
        assert_eq!(median, 50000.0);
    }
    
    #[test]
    fn test_mean_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_mean(&values), 3.0);
    }
    
    #[test]
    fn test_median_calculation() {
        let mut values = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        assert_eq!(calculate_median(&mut values), 3.0);
        
        let mut even_values = vec![5.0, 1.0, 3.0, 2.0];
        assert_eq!(calculate_median(&mut even_values), 2.5);
    }
}