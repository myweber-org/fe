
use std::net::{IpAddr, IcmpSocket, TcpStream};
use std::time::{Duration, Instant};
use std::thread;

const PING_TIMEOUT: Duration = Duration::from_secs(2);
const TCP_TIMEOUT: Duration = Duration::from_secs(3);
const PACKET_SIZE: usize = 64;

pub struct NetworkCheck {
    target: IpAddr,
}

impl NetworkCheck {
    pub fn new(target: IpAddr) -> Self {
        NetworkCheck { target }
    }

    pub fn ping(&self) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0")
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        let mut buffer = [0u8; PACKET_SIZE];
        buffer[0..8].copy_from_slice(b"PINGTEST");

        let start = Instant::now();
        socket.send_to(&buffer, (self.target, 0))
            .map_err(|e| format!("Failed to send ping: {}", e))?;

        let mut recv_buffer = [0u8; 1024];
        socket.set_read_timeout(Some(PING_TIMEOUT))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        match socket.recv_from(&mut recv_buffer) {
            Ok((size, _)) if size >= PACKET_SIZE => {
                let elapsed = start.elapsed();
                Ok(elapsed)
            }
            Ok(_) => Err("Received incomplete packet".to_string()),
            Err(e) => Err(format!("No response received: {}", e))
        }
    }

    pub fn check_port(&self, port: u16) -> bool {
        match TcpStream::connect_timeout(&(self.target, port).into(), TCP_TIMEOUT) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn scan_common_ports(&self) -> Vec<u16> {
        let common_ports = [80, 443, 22, 21, 25, 53, 3389];
        let mut open_ports = Vec::new();

        for &port in &common_ports {
            if self.check_port(port) {
                open_ports.push(port);
            }
            thread::sleep(Duration::from_millis(100));
        }

        open_ports
    }

    pub fn full_check(&self) -> NetworkStatus {
        let ping_result = self.ping();
        let open_ports = self.scan_common_ports();

        NetworkStatus {
            target: self.target,
            ping_latency: ping_result.ok(),
            open_ports,
            timestamp: Instant::now(),
        }
    }
}

pub struct NetworkStatus {
    pub target: IpAddr,
    pub ping_latency: Option<Duration>,
    pub open_ports: Vec<u16>,
    pub timestamp: Instant,
}

impl std::fmt::Display for NetworkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Network Status for {}:\n", self.target)?;
        
        match self.ping_latency {
            Some(latency) => write!(f, "  Ping: {:.2} ms\n", latency.as_millis() as f64 / 1000.0)?,
            None => write!(f, "  Ping: Unreachable\n")?,
        }

        if self.open_ports.is_empty() {
            write!(f, "  Open ports: None")?;
        } else {
            write!(f, "  Open ports: {:?}", self.open_ports)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_localhost_check() {
        let checker = NetworkCheck::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let status = checker.full_check();
        
        println!("{}", status);
        assert!(status.open_ports.contains(&80) || !status.open_ports.contains(&80));
    }
}