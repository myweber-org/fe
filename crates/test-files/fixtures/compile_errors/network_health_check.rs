use std::net::TcpStream;
use std::time::Duration;
use std::thread;
use std::io;

const RETRY_COUNT: u32 = 3;
const TIMEOUT_SECS: u64 = 5;

fn test_connection(host: &str, port: u16) -> io::Result<()> {
    match TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        Duration::from_secs(TIMEOUT_SECS)
    ) {
        Ok(_) => {
            println!("Successfully connected to {}:{}", host, port);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to connect to {}:{} - {}", host, port, e);
            Err(e)
        }
    }
}

fn main() {
    let test_host = "example.com";
    let test_port = 80;
    
    println!("Testing connection to {}:{}", test_host, test_port);
    
    for attempt in 1..=RETRY_COUNT {
        println!("Attempt {} of {}", attempt, RETRY_COUNT);
        
        match test_connection(test_host, test_port) {
            Ok(_) => {
                println!("Connection test passed");
                return;
            }
            Err(_) if attempt < RETRY_COUNT => {
                println!("Retrying in 2 seconds...");
                thread::sleep(Duration::from_secs(2));
            }
            Err(_) => {
                eprintln!("All connection attempts failed");
                std::process::exit(1);
            }
        }
    }
}