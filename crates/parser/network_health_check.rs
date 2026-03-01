use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::io::{self, Write};

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const PORTS_TO_CHECK: [u16; 3] = [80, 443, 8080];

fn test_connection(host: &str, port: u16) -> Result<(), String> {
    let start = Instant::now();
    
    match TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        CONNECTION_TIMEOUT
    ) {
        Ok(mut stream) => {
            let duration = start.elapsed();
            if let Err(e) = stream.write(b"PING") {
                return Err(format!("Write failed: {}", e));
            }
            println!("Connected to {}:{} in {:?}", host, port, duration);
            Ok(())
        }
        Err(e) => Err(format!("Connection failed: {}", e))
    }
}

fn check_network_health(host: &str) -> bool {
    println!("Testing connectivity to {}...", host);
    
    for port in &PORTS_TO_CHECK {
        let mut retries = 0;
        let mut last_error = String::new();
        
        while retries < MAX_RETRIES {
            match test_connection(host, *port) {
                Ok(_) => {
                    println!("Port {} is accessible", port);
                    break;
                }
                Err(e) => {
                    last_error = e;
                    retries += 1;
                    if retries < MAX_RETRIES {
                        println!("Retry {}/{} for port {}", retries, MAX_RETRIES, port);
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
        
        if retries == MAX_RETRIES {
            println!("Failed to connect to port {} after {} attempts: {}", 
                    port, MAX_RETRIES, last_error);
            return false;
        }
    }
    
    true
}

fn main() -> io::Result<()> {
    let test_hosts = ["google.com", "github.com", "rust-lang.org"];
    let mut all_healthy = true;
    
    for host in &test_hosts {
        if !check_network_health(host) {
            all_healthy = false;
            println!("{} has connectivity issues", host);
        }
        println!();
    }
    
    if all_healthy {
        println!("All network checks passed!");
    } else {
        println!("Some network checks failed");
    }
    
    Ok(())
}