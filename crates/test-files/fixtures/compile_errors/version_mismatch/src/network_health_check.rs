
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::io;
use std::thread;

const PING_TIMEOUT: Duration = Duration::from_secs(2);
const PORT_SCAN_TIMEOUT: Duration = Duration::from_secs(1);

pub struct NetworkHealth {
    host: String,
}

impl NetworkHealth {
    pub fn new(host: &str) -> Self {
        NetworkHealth {
            host: host.to_string(),
        }
    }

    pub fn ping(&self) -> Result<Duration, io::Error> {
        let start = Instant::now();
        let addr = format!("{}:80", self.host);
        
        match TcpStream::connect_timeout(
            &addr.to_socket_addrs()?.next().ok_or_else(|| 
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid hostname")
            )?,
            PING_TIMEOUT
        ) {
            Ok(_) => Ok(start.elapsed()),
            Err(e) => Err(e),
        }
    }

    pub fn scan_ports(&self, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start_port..=end_port {
            let host = self.host.clone();
            let handle = thread::spawn(move || {
                let addr = format!("{}:{}", host, port);
                match TcpStream::connect_timeout(
                    &addr.to_socket_addrs().unwrap().next().unwrap(),
                    PORT_SCAN_TIMEOUT
                ) {
                    Ok(_) => Some(port),
                    Err(_) => None,
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Some(port) = handle.join().unwrap() {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }

    pub fn check_http(&self) -> bool {
        self.scan_ports(80, 80).contains(&80) || 
        self.scan_ports(443, 443).contains(&443)
    }
}

pub fn run_health_check(host: &str) -> String {
    let checker = NetworkHealth::new(host);
    
    let mut result = String::new();
    result.push_str(&format!("Network Health Check for: {}\n", host));
    
    match checker.ping() {
        Ok(duration) => {
            result.push_str(&format!("Ping successful: {:.2?}\n", duration));
        }
        Err(e) => {
            result.push_str(&format!("Ping failed: {}\n", e));
        }
    }
    
    let open_ports = checker.scan_ports(20, 100);
    if !open_ports.is_empty() {
        result.push_str(&format!("Open ports: {:?}\n", open_ports));
    } else {
        result.push_str("No open ports found in range 20-100\n");
    }
    
    if checker.check_http() {
        result.push_str("HTTP/HTTPS service detected\n");
    } else {
        result.push_str("No HTTP/HTTPS service detected\n");
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_creation() {
        let checker = NetworkHealth::new("example.com");
        assert_eq!(checker.host, "example.com");
    }

    #[test]
    fn test_scan_ports_local() {
        let checker = NetworkHealth::new("127.0.0.1");
        let ports = checker.scan_ports(80, 85);
        assert!(ports.len() <= 6);
    }
}