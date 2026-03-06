use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkHealthChecker {
    timeout: Duration,
}

impl NetworkHealthChecker {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_host(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::TimedOut => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, bool)> {
        ports.iter()
            .map(|&port| (port, self.check_host(host, port).unwrap_or(false)))
            .collect()
    }
}

pub fn perform_health_check() -> String {
    let checker = NetworkHealthChecker::new(3);
    let test_host = "example.com";
    let test_ports = vec![80, 443, 22, 8080];
    
    let results = checker.scan_ports(test_host, &test_ports);
    
    let mut report = format!("Health check for {}:\n", test_host);
    for (port, status) in results {
        report.push_str(&format!("  Port {}: {}\n", port, 
            if status { "OPEN" } else { "CLOSED" }));
    }
    
    let all_open = results.iter().all(|(_, status)| *status);
    report.push_str(&format!("\nOverall status: {}", 
        if all_open { "HEALTHY" } else { "UNHEALTHY" }));
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check() {
        let checker = NetworkHealthChecker::new(1);
        // Localhost should have some closed ports
        let result = checker.check_host("127.0.0.1", 9999);
        assert!(result.is_ok());
        // Port 9999 is likely closed
        assert!(!result.unwrap());
    }
}