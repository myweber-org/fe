
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, PartialEq)]
enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
struct PacketHeader {
    source_ip: String,
    destination_ip: String,
    protocol: Protocol,
    payload_length: usize,
    timestamp: u64,
}

struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    suspicious_packets: Vec<PacketHeader>,
}

impl PacketAnalyzer {
    fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            suspicious_packets: Vec::new(),
        }
    }

    fn parse_protocol(&self, protocol_num: u8) -> Protocol {
        match protocol_num {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(protocol_num),
        }
    }

    fn analyze_packet(&mut self, raw_data: &[u8]) -> Option<PacketHeader> {
        if raw_data.len() < 20 {
            return None;
        }

        let version = (raw_data[0] >> 4) & 0x0F;
        
        let protocol_num = raw_data[9];
        let protocol = self.parse_protocol(protocol_num);
        
        let source_ip = match version {
            4 => {
                format!("{}.{}.{}.{}", 
                    raw_data[12], raw_data[13], 
                    raw_data[14], raw_data[15])
            }
            6 => {
                format!("{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                    raw_data[8], raw_data[9], raw_data[10], raw_data[11],
                    raw_data[12], raw_data[13], raw_data[14], raw_data[15])
            }
            _ => "Unknown".to_string(),
        };

        let dest_ip = match version {
            4 => {
                format!("{}.{}.{}.{}", 
                    raw_data[16], raw_data[17], 
                    raw_data[18], raw_data[19])
            }
            6 => {
                format!("{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                    raw_data[16], raw_data[17], raw_data[18], raw_data[19],
                    raw_data[20], raw_data[21], raw_data[22], raw_data[23])
            }
            _ => "Unknown".to_string(),
        };

        let header = PacketHeader {
            source_ip,
            destination_ip,
            protocol: protocol.clone(),
            payload_length: raw_data.len(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.packet_count += 1;
        *self.protocol_stats.entry(protocol).or_insert(0) += 1;

        if protocol_num == 1 || protocol_num == 58 {
            self.suspicious_packets.push(header.clone());
        }

        Some(header)
    }

    fn print_statistics(&self) {
        println!("Total packets analyzed: {}", self.packet_count);
        println!("Protocol statistics:");
        
        for (protocol, count) in &self.protocol_stats {
            println!("  {:?}: {}", protocol, count);
        }
        
        println!("Suspicious packets detected: {}", self.suspicious_packets.len());
    }
}

fn main() {
    let mut analyzer = PacketAnalyzer::new();
    
    let sample_tcp_packet = [
        0x45, 0x00, 0x00, 0x28, 0x00, 0x00, 0x40, 0x00,
        0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
        0xc0, 0xa8, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x50, 0x02, 0x20, 0x00,
        0x00, 0x00, 0x00, 0x00
    ];
    
    let sample_udp_packet = [
        0x45, 0x00, 0x00, 0x1c, 0x00, 0x00, 0x40, 0x00,
        0x40, 0x11, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
        0xc0, 0xa8, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00
    ];
    
    if let Some(header) = analyzer.analyze_packet(&sample_tcp_packet) {
        println!("Analyzed TCP packet: {:?}", header);
    }
    
    if let Some(header) = analyzer.analyze_packet(&sample_udp_packet) {
        println!("Analyzed UDP packet: {:?}", header);
    }
    
    analyzer.print_statistics();
}