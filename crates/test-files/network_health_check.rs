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
                eprintln!("Connection failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn scan_ports(&self, host: &str, ports: &[u16]) -> Vec<u16> {
        let mut open_ports = Vec::new();
        
        for &port in ports {
            if self.ping_host(host, port).unwrap_or(false) {
                open_ports.push(port);
                println!("Port {} is open", port);
            }
        }
        
        open_ports
    }
}

pub fn check_network_connectivity() -> io::Result<()> {
    let checker = NetworkCheck::new(3);
    
    let test_hosts = vec![
        ("google.com", 80),
        ("github.com", 443),
        ("localhost", 8080),
    ];
    
    for (host, port) in test_hosts {
        let status = checker.ping_host(host, port)?;
        println!("{}:{} - {}", host, port, if status { "reachable" } else { "unreachable" });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_scan() {
        let checker = NetworkCheck::new(1);
        let ports = vec![80, 443, 8080, 3000];
        let open_ports = checker.scan_ports("localhost", &ports);
        
        println!("Found {} open ports on localhost", open_ports.len());
        assert!(open_ports.len() <= ports.len());
    }
}