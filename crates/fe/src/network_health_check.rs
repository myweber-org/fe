use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;

fn check_connectivity(host: &str, port: u16, timeout_secs: u64) -> bool {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address format");
    
    for attempt in 1..=3 {
        println!("Attempt {} to connect to {}:{}", attempt, host, port);
        
        match TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
            Ok(stream) => {
                println!("Successfully connected to {}:{}", host, port);
                drop(stream);
                return true;
            }
            Err(e) => {
                println!("Connection failed: {}", e);
                if attempt < 3 {
                    println!("Retrying in 2 seconds...");
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }
    
    false
}

fn main() {
    let test_host = "example.com";
    let test_port = 80;
    let timeout = 5;
    
    if check_connectivity(test_host, test_port, timeout) {
        println!("Network connectivity test PASSED");
    } else {
        println!("Network connectivity test FAILED");
    }
}