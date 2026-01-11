use std::net::UdpSocket;
use std::time::Duration;

#[derive(Debug)]
struct PacketHeader {
    source_port: u16,
    destination_port: u16,
    length: u16,
    checksum: u16,
}

impl PacketHeader {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        
        Some(PacketHeader {
            source_port: u16::from_be_bytes([data[0], data[1]]),
            destination_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        })
    }
    
    fn validate_checksum(&self, payload: &[u8]) -> bool {
        let mut sum: u32 = 0;
        
        sum += self.source_port as u32;
        sum += self.destination_port as u32;
        sum += self.length as u32;
        
        for chunk in payload.chunks(2) {
            let word = if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]]) as u32
            } else {
                (chunk[0] as u32) << 8
            };
            sum += word;
        }
        
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        let checksum = !sum as u16;
        checksum == self.checksum
    }
}

struct PacketAnalyzer {
    socket: UdpSocket,
    packet_count: u32,
    total_bytes: u64,
}

impl PacketAnalyzer {
    fn new(bind_addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        
        Ok(PacketAnalyzer {
            socket,
            packet_count: 0,
            total_bytes: 0,
        })
    }
    
    fn capture_packets(&mut self, max_packets: u32) -> std::io::Result<()> {
        let mut buffer = [0u8; 65535];
        
        println!("Starting packet capture on {}", self.socket.local_addr()?);
        
        while self.packet_count < max_packets {
            match self.socket.recv_from(&mut buffer) {
                Ok((size, source)) => {
                    self.packet_count += 1;
                    self.total_bytes += size as u64;
                    
                    if let Some(header) = PacketHeader::from_bytes(&buffer[..8]) {
                        let payload = &buffer[8..size];
                        let valid = header.validate_checksum(payload);
                        
                        println!("Packet #{} from {}:{} to {}:{} ({} bytes, checksum {})",
                            self.packet_count,
                            source.ip(), header.source_port,
                            source.ip(), header.destination_port,
                            size,
                            if valid { "valid" } else { "invalid" }
                        );
                    } else {
                        println!("Packet #{}: Invalid header format", self.packet_count);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    fn print_statistics(&self) {
        println!("\nCapture Statistics:");
        println!("  Total packets: {}", self.packet_count);
        println!("  Total bytes: {}", self.total_bytes);
        if self.packet_count > 0 {
            println!("  Average packet size: {:.2} bytes", 
                self.total_bytes as f64 / self.packet_count as f64);
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut analyzer = PacketAnalyzer::new("127.0.0.1:0")?;
    
    analyzer.capture_packets(10)?;
    analyzer.print_statistics();
    
    Ok(())
}