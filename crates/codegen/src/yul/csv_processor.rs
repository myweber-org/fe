use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn filter_csv(input_path: &str, output_path: &str, column_index: usize, filter_value: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let columns: Vec<&str> = line.split(',').collect();
        
        if columns.get(column_index).map_or(false, |&val| val == filter_value) {
            writeln!(output_file, "{}", line)?;
        }
    }

    Ok(())
}

pub fn transform_column(input_path: &str, output_path: &str, column_index: usize, transform_fn: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let mut columns: Vec<&str> = line.split(',').collect();
        
        if let Some(cell) = columns.get_mut(column_index) {
            *cell = &transform_fn(cell);
        }
        
        writeln!(output_file, "{}", columns.join(","))?;
    }

    Ok(())
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_filter_csv() -> Result<(), Box<dyn Error>> {
        let test_data = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Charlie,active";
        fs::write("test_input.csv", test_data)?;
        
        filter_csv("test_input.csv", "test_output.csv", 2, "active")?;
        
        let output = fs::read_to_string("test_output.csv")?;
        assert!(output.contains("Alice"));
        assert!(!output.contains("Bob"));
        assert!(output.contains("Charlie"));
        
        fs::remove_file("test_input.csv")?;
        fs::remove_file("test_output.csv")?;
        Ok(())
    }

    #[test]
    fn test_transform_column() -> Result<(), Box<dyn Error>> {
        let test_data = "id,name,status\n1,alice,active\n2,bob,inactive";
        fs::write("test_transform_input.csv", test_data)?;
        
        transform_column("test_transform_input.csv", "test_transform_output.csv", 1, uppercase_transform)?;
        
        let output = fs::read_to_string("test_transform_output.csv")?;
        assert!(output.contains("ALICE"));
        assert!(output.contains("BOB"));
        
        fs::remove_file("test_transform_input.csv")?;
        fs::remove_file("test_transform_output.csv")?;
        Ok(())
    }
}