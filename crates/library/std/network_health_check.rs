use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};
use std::io::{self, Write};

const PACKET_SIZE: usize = 64;
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PACKETS: usize = 10;

pub struct NetworkHealth {
    target_host: String,
    target_port: u16,
}

impl NetworkHealth {
    pub fn new(host: &str, port: u16) -> Self {
        NetworkHealth {
            target_host: host.to_string(),
            target_port: port,
        }
    }

    pub fn check_tcp_connectivity(&self) -> io::Result<Duration> {
        let start = Instant::now();
        let stream = TcpStream::connect_timeout(
            &format!("{}:{}", self.target_host, self.target_port).parse().unwrap(),
            TIMEOUT,
        )?;
        let duration = start.elapsed();
        drop(stream);
        Ok(duration)
    }

    pub fn measure_packet_loss(&self) -> (f64, Duration) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();

        let mut packets_sent = 0;
        let mut packets_received = 0;
        let mut total_latency = Duration::default();
        let test_data = [0u8; PACKET_SIZE];

        while packets_sent < MAX_PACKETS {
            let send_time = Instant::now();
            let send_result = socket.send_to(&test_data, (self.target_host.as_str(), self.target_port));
            
            if send_result.is_ok() {
                packets_sent += 1;
                
                let mut buffer = [0u8; PACKET_SIZE];
                match socket.recv_from(&mut buffer) {
                    Ok((size, _)) if size == PACKET_SIZE => {
                        packets_received += 1;
                        total_latency += send_time.elapsed();
                    }
                    _ => {}
                }
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }

        let packet_loss = if packets_sent > 0 {
            (1.0 - (packets_received as f64 / packets_sent as f64)) * 100.0
        } else {
            100.0
        };

        let avg_latency = if packets_received > 0 {
            total_latency / packets_received as u32
        } else {
            Duration::default()
        };

        (packet_loss, avg_latency)
    }

    pub fn run_diagnostics(&self) {
        println!("Running network diagnostics for {}:{}", self.target_host, self.target_port);
        
        match self.check_tcp_connectivity() {
            Ok(latency) => println!("TCP connectivity: OK ({} ms)", latency.as_millis()),
            Err(e) => println!("TCP connectivity: FAILED ({})", e),
        }

        let (packet_loss, avg_latency) = self.measure_packet_loss();
        println!("UDP packet loss: {:.1}%", packet_loss);
        println!("Average UDP latency: {} ms", avg_latency.as_millis());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_creation() {
        let checker = NetworkHealth::new("example.com", 80);
        assert_eq!(checker.target_host, "example.com");
        assert_eq!(checker.target_port, 80);
    }
}