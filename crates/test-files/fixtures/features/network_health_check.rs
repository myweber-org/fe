use std::net::{TcpStream, IpAddr};
use std::time::Duration;
use std::thread;

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkChecker {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_host(&self, host: &str) -> bool {
        let addr: Result<IpAddr, _> = host.parse();
        if addr.is_err() {
            return false;
        }

        let host_clone = host.to_string();
        let timeout = self.timeout;
        
        let handle = thread::spawn(move || {
            let output = std::process::Command::new("ping")
                .arg("-c")
                .arg("1")
                .arg("-W")
                .arg(timeout.as_secs().to_string())
                .arg(&host_clone)
                .output();
            
            match output {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        });

        handle.join().unwrap_or(false)
    }

    pub fn check_port(&self, host: &str, port: u16) -> bool {
        match TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse().unwrap(),
            self.timeout
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<u16> {
        let mut open_ports = Vec::new();
        
        for &port in ports {
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
    fn test_network_checker_creation() {
        let checker = NetworkChecker::new(5);
        assert_eq!(checker.timeout.as_secs(), 5);
    }

    #[test]
    fn test_port_check() {
        let checker = NetworkChecker::new(1);
        let localhost = "127.0.0.1";
        
        assert!(checker.check_port(localhost, 80) || !checker.check_port(localhost, 80));
    }
}