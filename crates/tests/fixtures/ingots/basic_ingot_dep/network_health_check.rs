
use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const PORTS_TO_CHECK: [u16; 3] = [80, 443, 53];

pub struct NetworkHealth {
    pub host: String,
    pub timeout: Duration,
}

impl NetworkHealth {
    pub fn new(host: &str) -> Self {
        NetworkHealth {
            host: host.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn check_connectivity(&self) -> io::Result<Vec<(u16, bool, Duration)>> {
        let mut results = Vec::new();

        for &port in &PORTS_TO_CHECK {
            let addr = format!("{}:{}", self.host, port);
            let socket_addr: SocketAddr = match addr.parse() {
                Ok(addr) => addr,
                Err(_) => continue,
            };

            let start = Instant::now();
            let connection_result = TcpStream::connect_timeout(&socket_addr, self.timeout);
            let duration = start.elapsed();

            let success = connection_result.is_ok();
            results.push((port, success, duration));
        }

        Ok(results)
    }

    pub fn print_report(&self) -> io::Result<()> {
        let results = self.check_connectivity()?;
        
        println!("Network Health Report for: {}", self.host);
        println!("{:-<40}", "");
        
        for (port, success, duration) in results {
            let status = if success { "✓" } else { "✗" };
            println!("Port {:4}: {} ({} ms)", port, status, duration.as_millis());
        }
        
        let successful_checks = results.iter().filter(|(_, success, _)| *success).count();
        println!("\nSummary: {}/{} ports reachable", successful_checks, PORTS_TO_CHECK.len());
        
        Ok(())
    }
}

pub fn quick_check(host: &str) -> bool {
    let checker = NetworkHealth::new(host);
    match checker.check_connectivity() {
        Ok(results) => results.iter().any(|(_, success, _)| *success),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_creation() {
        let checker = NetworkHealth::new("example.com");
        assert_eq!(checker.host, "example.com");
        assert_eq!(checker.timeout, DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_with_timeout() {
        let custom_timeout = Duration::from_secs(2);
        let checker = NetworkHealth::new("example.com").with_timeout(custom_timeout);
        assert_eq!(checker.timeout, custom_timeout);
    }
}