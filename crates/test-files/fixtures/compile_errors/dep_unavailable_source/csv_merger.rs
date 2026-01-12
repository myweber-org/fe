use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn merge_csv_files(input_files: Vec<String>, output_file: &str) -> io::Result<()> {
    if input_files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No input files provided",
        ));
    }

    let mut output = File::create(output_file)?;
    let mut headers_written = false;

    for (index, filename) in input_files.iter().enumerate() {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header = header_result?;

            if index == 0 {
                writeln!(output, "{}", header)?;
                headers_written = true;
            } else if !headers_written {
                writeln!(output, "{}", header)?;
                headers_written = true;
            }

            for line in lines {
                let line_content = line?;
                if !line_content.trim().is_empty() {
                    writeln!(output, "{}", line_content)?;
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <output.csv> <input1.csv> <input2.csv> ...", args[0]);
        std::process::exit(1);
    }

    let output_file = &args[1];
    let input_files: Vec<String> = args[2..].to_vec();

    match merge_csv_files(input_files, output_file) {
        Ok(()) => {
            println!("Successfully merged files into {}", output_file);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error merging files: {}", e);
            Err(e)
        }
    }
}