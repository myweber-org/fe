use std::net::TcpStream;
use std::time::Duration;
use std::thread;

const HOST: &str = "example.com";
const PORT: u16 = 80;
const MAX_RETRIES: u8 = 3;
const TIMEOUT_SECS: u64 = 5;

fn test_connection(host: &str, port: u16) -> bool {
    match TcpStream::connect((host, port)) {
        Ok(_) => {
            println!("Successfully connected to {}:{}", host, port);
            true
        }
        Err(e) => {
            eprintln!("Connection failed: {}", e);
            false
        }
    }
}

fn main() {
    let mut attempts = 0;
    
    while attempts < MAX_RETRIES {
        attempts += 1;
        println!("Attempt {} of {}", attempts, MAX_RETRIES);
        
        let result = thread::scope(|s| {
            s.spawn(|| test_connection(HOST, PORT))
                .join()
                .unwrap_or_else(|_| {
                    eprintln!("Thread panicked during connection test");
                    false
                })
        });
        
        if result {
            println!("Network connectivity verified");
            return;
        }
        
        if attempts < MAX_RETRIES {
            println!("Waiting {} seconds before retry...", TIMEOUT_SECS);
            thread::sleep(Duration::from_secs(TIMEOUT_SECS));
        }
    }
    
    eprintln!("Failed to establish connection after {} attempts", MAX_RETRIES);
    std::process::exit(1);
}