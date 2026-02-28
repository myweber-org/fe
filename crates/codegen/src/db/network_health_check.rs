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
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::net::UdpSocket;
use tokio::time::sleep;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const MAX_PACKETS: u32 = 10;

pub struct NetworkProbe {
    target: SocketAddr,
    packet_loss: Arc<AtomicU32>,
    avg_latency: Arc<AtomicU32>,
}

impl NetworkProbe {
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        let target = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
            port,
        );
        NetworkProbe {
            target,
            packet_loss: Arc::new(AtomicU32::new(0)),
            avg_latency: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn start_monitoring(&self) {
        let loss_counter = self.packet_loss.clone();
        let latency_counter = self.avg_latency.clone();
        let target = self.target;

        tokio::spawn(async move {
            let mut successful_packets = 0;
            let mut total_latency = Duration::from_millis(0);

            for seq in 0..MAX_PACKETS {
                if let Ok(latency) = Self::send_probe_packet(target, seq).await {
                    successful_packets += 1;
                    total_latency += latency;
                } else {
                    loss_counter.fetch_add(1, Ordering::Relaxed);
                }
                sleep(Duration::from_millis(500)).await;
            }

            if successful_packets > 0 {
                let avg_ms = total_latency.as_millis() / successful_packets as u128;
                latency_counter.store(avg_ms as u32, Ordering::Relaxed);
            }
        });
    }

    async fn send_probe_packet(target: SocketAddr, sequence: u32) -> Result<Duration, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))?;

        let mut buffer = [0u8; PACKET_SIZE];
        buffer[0..4].copy_from_slice(&sequence.to_be_bytes());

        let start = Instant::now();
        socket.send_to(&buffer, target).await?;

        let mut recv_buffer = [0u8; PACKET_SIZE];
        let (size, _) = socket.recv_from(&mut recv_buffer).await?;

        if size >= 4 {
            let end = Instant::now();
            let latency = end.duration_since(start);
            return Ok(latency);
        }

        Err("Invalid response packet".into())
    }

    pub fn get_stats(&self) -> (u32, u32) {
        let loss = self.packet_loss.load(Ordering::Relaxed);
        let latency = self.avg_latency.load(Ordering::Relaxed);
        (loss, latency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probe_creation() {
        let probe = NetworkProbe::new([8, 8, 8, 8], 53);
        assert_eq!(probe.target.port(), 53);
    }
}