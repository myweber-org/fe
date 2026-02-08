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
    use tempfile::NamedTempFile;

    #[test]
    fn test_merge_csv_files() {
        let csv1 = "id,name\n1,Alice\n2,Bob";
        let csv2 = "id,name\n3,Charlie\n4,Diana";

        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        fs::write(file1.path(), csv1).unwrap();
        fs::write(file2.path(), csv2).unwrap();

        let input_paths = vec![
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];

        merge_csv_files(&input_paths, output_file.path().to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name\n1,Alice\n2,Bob\n3,Charlie\n4,Diana\n";
        assert_eq!(content, expected);
    }
}