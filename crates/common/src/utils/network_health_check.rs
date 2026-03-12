use std::net::{IpAddr, IcmpSocket, TcpStream};
use std::time::{Duration, Instant};
use std::thread;

pub struct NetworkCheck {
    timeout: Duration,
}

impl NetworkCheck {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkCheck {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn ping(&self, target: IpAddr) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind ICMP socket: {}", e))?;
        
        socket.set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let start = Instant::now();
        let mut buffer = [0u8; 64];
        
        match socket.send_to(&buffer, (target, 0)) {
            Ok(_) => {
                let mut recv_buffer = [0u8; 1024];
                match socket.recv_from(&mut recv_buffer) {
                    Ok(_) => {
                        let elapsed = start.elapsed();
                        Ok(elapsed)
                    },
                    Err(e) => Err(format!("No response received: {}", e))
                }
            },
            Err(e) => Err(format!("Failed to send ping: {}", e))
        }
    }

    pub fn check_port(&self, target: IpAddr, port: u16) -> bool {
        match TcpStream::connect_timeout(&(target, port).into(), self.timeout) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn scan_ports(&self, target: IpAddr, start_port: u16, end_port: u16) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let mut handles = Vec::new();

        for port in start_port..=end_port {
            let timeout = self.timeout;
            let target_clone = target;
            let handle = thread::spawn(move || {
                match TcpStream::connect_timeout(&(target_clone, port).into(), timeout) {
                    Ok(_) => Some(port),
                    Err(_) => None
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Ok(Some(port)) = handle.join() {
                open_ports.push(port);
            }
        }

        open_ports.sort();
        open_ports
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_localhost_port_check() {
        let checker = NetworkCheck::new(2);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        // Port 80 should be closed on localhost unless web server running
        assert!(!checker.check_port(localhost, 80));
    }

    #[test]
    fn test_port_scan_range() {
        let checker = NetworkCheck::new(1);
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        
        let open_ports = checker.scan_ports(localhost, 80, 85);
        // Should return empty vector or specific ports depending on system
        println!("Found open ports on localhost: {:?}", open_ports);
    }
}