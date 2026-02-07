use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut headers_written = false;

    for input_path in input_paths {
        let path = Path::new(input_path);
        let mut rdr = csv::Reader::from_path(path)?;
        let headers = rdr.headers()?.clone();

        if !headers_written {
            writer.write_all(headers.as_bytes())?;
            writer.write_all(b"\n")?;
            headers_written = true;
        }

        for result in rdr.records() {
            let record = result?;
            writer.write_all(record.as_slice().as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_csv_files() {
        let test_dir = "test_data";
        fs::create_dir_all(test_dir).unwrap();

        let file1_content = "id,name\n1,Alice\n2,Bob";
        let file2_content = "id,name\n3,Charlie\n4,Diana";

        let file1_path = format!("{}/file1.csv", test_dir);
        let file2_path = format!("{}/file2.csv", test_dir);
        let output_path = format!("{}/merged.csv", test_dir);

        fs::write(&file1_path, file1_content).unwrap();
        fs::write(&file2_path, file2_content).unwrap();

        let inputs = [file1_path.as_str(), file2_path.as_str()];
        merge_csv_files(&inputs, &output_path).unwrap();

        let merged_content = fs::read_to_string(&output_path).unwrap();
        let expected = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,Diana\n";
        assert_eq!(merged_content, expected);

        fs::remove_dir_all(test_dir).unwrap();
    }
}