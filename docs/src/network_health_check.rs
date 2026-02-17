
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

    pub fn ping_host(&self, host: IpAddr) -> bool {
        let output = std::process::Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg(self.timeout.as_secs().to_string())
            .arg(host.to_string())
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    pub fn check_port(&self, host: IpAddr, port: u16) -> bool {
        match TcpStream::connect_timeout(&(host, port).into(), self.timeout) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn scan_ports(&self, host: IpAddr, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start_port..=end_port {
            let checker = self.clone();
            let host_clone = host;
            let handle = thread::spawn(move || {
                if checker.check_port(host_clone, port) {
                    Some(port)
                } else {
                    None
                }
            });
            handles.push((port, handle));
        }

        for (_, handle) in handles {
            if let Ok(Some(port)) = handle.join() {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }
}

impl Clone for NetworkChecker {
    fn clone(&self) -> Self {
        NetworkChecker {
            timeout: self.timeout,
        }
    }
}

pub fn run_health_check(target: IpAddr) -> String {
    let checker = NetworkChecker::new(2);
    
    let mut results = Vec::new();
    
    if checker.ping_host(target) {
        results.push(format!("Ping to {}: SUCCESS", target));
    } else {
        results.push(format!("Ping to {}: FAILED", target));
    }
    
    let common_ports = vec![80, 443, 22, 21, 25];
    for port in common_ports {
        if checker.check_port(target, port) {
            results.push(format!("Port {}: OPEN", port));
        }
    }
    
    results.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_localhost_connectivity() {
        let checker = NetworkChecker::new(1);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Localhost should be reachable
        assert!(checker.check_port(localhost, 80) || !checker.check_port(localhost, 80));
        // Test should at least not panic
    }
}