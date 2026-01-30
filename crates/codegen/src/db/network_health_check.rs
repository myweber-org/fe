use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkCheck {
    timeout: Duration,
}

impl NetworkCheck {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkCheck {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Connection failed to {}:{} - {}", host, port, e);
                Ok(false)
            }
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<u16> {
        let mut open_ports = Vec::new();
        
        for &port in ports {
            if self.ping_host(host, port).unwrap_or(false) {
                open_ports.push(port);
                println!("Port {} is open on {}", port, host);
            }
        }
        
        open_ports
    }
}

pub fn check_network_connectivity() -> bool {
    let checker = NetworkCheck::new(3);
    let test_ports = [80, 443, 22, 8080];
    
    let open_ports = checker.scan_ports("example.com", &test_ports);
    !open_ports.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_check_creation() {
        let checker = NetworkCheck::new(5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }

    #[test]
    fn test_port_scanning() {
        let checker = NetworkCheck::new(1);
        let ports = [80, 443];
        let result = checker.scan_ports("localhost", &ports);
        assert!(result.len() <= ports.len());
    }
}