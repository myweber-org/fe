use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    if input_paths.is_empty() {
        return Err("No input files provided".into());
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for (index, input_path) in input_paths.iter().enumerate() {
        let path = Path::new(input_path);
        if !path.exists() {
            return Err(format!("Input file not found: {}", input_path).into());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            let header = first_line?;

            if index == 0 {
                writer.write_all(header.as_bytes())?;
                writer.write_all(b"\n")?;
                headers_written = true;
            } else if header != get_first_line(input_paths[0])? {
                eprintln!("Warning: Header mismatch between files. Using header from first file.");
            }

            for line in lines {
                let line = line?;
                if !line.trim().is_empty() {
                    writer.write_all(line.as_bytes())?;
                    writer.write_all(b"\n")?;
                }
            }
        }
    }

    writer.flush()?;
    println!("Successfully merged {} files into {}", input_paths.len(), output_path);
    Ok(())
}

fn get_first_line(file_path: &str) -> Result<String, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let first_line = reader.lines().next().ok_or("File is empty")??;
    Ok(first_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let test_dir = "test_csv_merge";
        fs::create_dir_all(test_dir).unwrap();

        let file1_content = "id,name,value\n1,Alice,100\n2,Bob,200";
        let file2_content = "id,name,value\n3,Charlie,300\n4,David,400";

        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        let inputs = [file1_path.as_str(), file2_path.as_str()];
        let result = merge_csv_files(&inputs, &output_path);

        assert!(result.is_ok());
        let merged_content = fs::read_to_string(&output_path).unwrap();
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n4,David,400\n";
        assert_eq!(merged_content, expected);

        fs::remove_dir_all(test_dir).unwrap();
    }
}