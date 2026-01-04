
use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(data: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    data.into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_removes_duplicates() {
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let result = clean_data(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_clean_data_preserves_order() {
        let input = vec!["apple", "banana", "apple", "cherry"];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_clean_data_empty_input() {
        let input: Vec<i32> = vec![];
        let result = clean_data(input);
        assert!(result.is_empty());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            writeln!(output_file, "{}", line)?;
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 3 && !fields[1].is_empty() && fields[2].parse::<f64>().is_ok() {
            writeln!(output_file, "{}", line)?;
        }
    }

    Ok(())
}