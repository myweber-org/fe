
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(Path::new(output_path))?;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty() {
            let cleaned_columns: Vec<String> = trimmed_line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();

            if cleaned_columns.iter().any(|col| !col.is_empty()) {
                writeln!(output_file, "{}", cleaned_columns.join(","))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_clean_csv() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";

        let test_data = "name,age,city\nJohn,25,NYC\n\n  ,  ,  \nAlice,30,London\n\n";
        fs::write(test_input, test_data).unwrap();

        clean_csv(test_input, test_output).unwrap();

        let result = fs::read_to_string(test_output).unwrap();
        let expected = "name,age,city\nJohn,25,NYC\nAlice,30,London\n";

        assert_eq!(result, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}