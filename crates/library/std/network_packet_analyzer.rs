use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

#[derive(Debug)]
struct PacketInfo {
    source: SocketAddrV4,
    destination: SocketAddrV4,
    payload_size: usize,
    timestamp: std::time::Instant,
}

struct PacketAnalyzer {
    socket: UdpSocket,
    packet_buffer: [u8; 1024],
    received_packets: Vec<PacketInfo>,
}

impl PacketAnalyzer {
    fn new(port: u16) -> std::io::Result<Self> {
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        
        Ok(PacketAnalyzer {
            socket,
            packet_buffer: [0; 1024],
            received_packets: Vec::new(),
        })
    }

    fn capture_packets(&mut self, count: usize) -> std::io::Result<()> {
        for _ in 0..count {
            match self.socket.recv_from(&mut self.packet_buffer) {
                Ok((size, source_addr)) => {
                    if let std::net::SocketAddr::V4(source) = source_addr {
                        let packet_info = PacketInfo {
                            source,
                            destination: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.socket.local_addr()?.port()),
                            payload_size: size,
                            timestamp: std::time::Instant::now(),
                        };
                        self.received_packets.push(packet_info);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn generate_statistics(&self) -> AnalysisReport {
        let total_packets = self.received_packets.len();
        let total_bytes: usize = self.received_packets.iter().map(|p| p.payload_size).sum();
        let avg_packet_size = if total_packets > 0 {
            total_bytes / total_packets
        } else {
            0
        };

        AnalysisReport {
            total_packets,
            total_bytes,
            avg_packet_size,
            capture_duration: self.received_packets.last()
                .and_then(|last| self.received_packets.first()
                    .map(|first| last.timestamp.duration_since(first.timestamp)))
                .unwrap_or(Duration::from_secs(0)),
        }
    }

    fn display_results(&self) {
        let stats = self.generate_statistics();
        println!("Packet Analysis Results:");
        println!("Total packets captured: {}", stats.total_packets);
        println!("Total bytes received: {}", stats.total_bytes);
        println!("Average packet size: {} bytes", stats.avg_packet_size);
        println!("Capture duration: {:?}", stats.capture_duration);
        
        if !self.received_packets.is_empty() {
            println!("\nFirst 5 packets:");
            for (i, packet) in self.received_packets.iter().take(5).enumerate() {
                println!("  Packet {}: from {} ({} bytes)", 
                    i + 1, packet.source, packet.payload_size);
            }
        }
    }
}

#[derive(Debug)]
struct AnalysisReport {
    total_packets: usize,
    total_bytes: usize,
    avg_packet_size: usize,
    capture_duration: Duration,
}

fn main() -> std::io::Result<()> {
    let mut analyzer = PacketAnalyzer::new(8080)?;
    println!("Starting packet capture on port 8080...");
    
    analyzer.capture_packets(50)?;
    analyzer.display_results();
    
    Ok(())
}