use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct NetworkHealthChecker {
    timeout: Duration,
}

impl NetworkHealthChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkHealthChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_host(&self, host: &str, port: u16) -> Result<bool, String> {
        let addr_string = format!("{}:{}", host, port);
        let addrs: Vec<_> = addr_string.to_socket_addrs()
            .map_err(|e| format!("DNS resolution failed: {}", e))?
            .collect();

        if addrs.is_empty() {
            return Err("No addresses resolved".to_string());
        }

        for addr in addrs {
            match TcpStream::connect_timeout(&addr, self.timeout) {
                Ok(_) => return Ok(true),
                Err(e) => {
                    if addr == *addrs.last().unwrap() {
                        return Err(format!("Connection failed to all addresses: {}", e));
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn check_multiple_ports(&self, host: &str, ports: &[u16]) -> Vec<(u16, Result<bool, String>)> {
        ports.iter()
            .map(|&port| (port, self.check_host(host, port)))
            .collect()
    }
}

pub fn perform_health_check() {
    let checker = NetworkHealthChecker::new(5);
    let target_host = "example.com";
    let ports = vec![80, 443, 22, 8080];

    println!("Checking network health for {}...", target_host);
    
    for (port, result) in checker.check_multiple_ports(target_host, &ports) {
        match result {
            Ok(true) => println!("Port {}: OPEN", port),
            Ok(false) => println!("Port {}: CLOSED", port),
            Err(e) => println!("Port {}: ERROR - {}", port, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connection() {
        let checker = NetworkHealthChecker::new(2);
        let result = checker.check_host("localhost", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_host() {
        let checker = NetworkHealthChecker::new(1);
        let result = checker.check_host("invalid.host.that.does.not.exist", 80);
        assert!(result.is_err());
    }
}