use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const MAX_RETRIES: u32 = 3;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const PORTS_TO_CHECK: [u16; 3] = [80, 443, 8080];

fn check_port(host: &str, port: u16) -> Result<Duration, String> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    
    let start = Instant::now();
    match TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT) {
        Ok(_) => {
            let elapsed = start.elapsed();
            drop(stream);
            Ok(elapsed)
        }
        Err(e) => Err(format!("Failed to connect to {}:{} - {}", host, port, e)),
    }
}

fn perform_health_check(host: &str) -> Vec<(u16, Result<Duration, String>)> {
    PORTS_TO_CHECK
        .iter()
        .map(|&port| {
            let mut last_error = None;
            for attempt in 1..=MAX_RETRIES {
                match check_port(host, port) {
                    Ok(duration) => return (port, Ok(duration)),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < MAX_RETRIES {
                            std::thread::sleep(Duration::from_millis(200 * attempt as u64));
                        }
                    }
                }
            }
            (port, Err(last_error.unwrap_or_else(|| "Unknown error".to_string())))
        })
        .collect()
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1 {
        "<1ms".to_string()
    } else {
        format!("{}ms", millis)
    }
}

fn main() -> io::Result<()> {
    let host = "example.com";
    println!("Performing network health check for {}...", host);
    
    let results = perform_health_check(host);
    
    let mut all_successful = true;
    for (port, result) in results {
        match result {
            Ok(duration) => {
                println!("Port {}: OK (response time: {})", port, format_duration(duration));
            }
            Err(e) => {
                println!("Port {}: FAILED - {}", port, e);
                all_successful = false;
            }
        }
    }
    
    if all_successful {
        println!("All required ports are accessible.");
        Ok(())
    } else {
        println!("Some ports failed connectivity checks.");
        std::process::exit(1);
    }
}