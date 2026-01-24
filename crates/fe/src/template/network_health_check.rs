use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct NetworkCheck {
    timeout: Duration,
}

impl NetworkCheck {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkCheck {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping_host(&self, host: &str) -> bool {
        let addr = match host.to_socket_addrs() {
            Ok(mut addrs) => addrs.next(),
            Err(_) => return false,
        };

        if let Some(addr) = addr {
            match TcpStream::connect_timeout(&addr, self.timeout) {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> bool {
        let addr_string = format!("{}:{}", host, port);
        match TcpStream::connect_timeout(&addr_string.parse().unwrap(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: &str, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        
        for port in start_port..=end_port {
            if self.check_port(host, port) {
                open_ports.push(port);
            }
        }
        
        open_ports
    }
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
    fn test_ping_localhost() {
        let checker = NetworkCheck::new(2);
        assert!(checker.ping_host("localhost"));
    }
}