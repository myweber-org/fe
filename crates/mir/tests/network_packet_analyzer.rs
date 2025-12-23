use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub source_ip: Ipv4Addr,
    pub destination_ip: Ipv4Addr,
    pub protocol: Protocol,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    suspicious_packets: Vec<NetworkPacket>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            suspicious_packets: Vec::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: NetworkPacket) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(packet.protocol.clone()).or_insert(0) += 1;
        
        if Self::is_suspicious(&packet) {
            self.suspicious_packets.push(packet.clone());
            println!("Suspicious packet detected from {} to {}", 
                     packet.source_ip, packet.destination_ip);
        }
    }

    fn is_suspicious(packet: &NetworkPacket) -> bool {
        match packet.protocol {
            Protocol::ICMP => packet.payload.len() > 1024,
            Protocol::TCP => {
                let suspicious_ports = [21, 22, 23, 25, 80, 443];
                let dest_port = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
                suspicious_ports.contains(&dest_port) && packet.payload.len() < 20
            }
            _ => false,
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_packets".to_string(), self.packet_count);
        
        for (protocol, count) in &self.protocol_stats {
            let protocol_name = match protocol {
                Protocol::TCP => "tcp",
                Protocol::UDP => "udp",
                Protocol::ICMP => "icmp",
                Protocol::Unknown(_) => "unknown",
            };
            stats.insert(format!("protocol_{}", protocol_name), *count);
        }
        
        stats.insert("suspicious_packets".to_string(), self.suspicious_packets.len() as u64);
        stats
    }

    pub fn detect_protocol(protocol_number: u8) -> Protocol {
        match protocol_number {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            n => Protocol::Unknown(n),
        }
    }
}

impl Default for PacketAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_detection() {
        assert_eq!(PacketAnalyzer::detect_protocol(6), Protocol::TCP);
        assert_eq!(PacketAnalyzer::detect_protocol(17), Protocol::UDP);
        assert_eq!(PacketAnalyzer::detect_protocol(1), Protocol::ICMP);
        assert_eq!(PacketAnalyzer::detect_protocol(99), Protocol::Unknown(99));
    }

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet = NetworkPacket {
            source_ip: Ipv4Addr::new(192, 168, 1, 1),
            destination_ip: Ipv4Addr::new(192, 168, 1, 2),
            protocol: Protocol::TCP,
            payload: vec![0, 80, 1, 2, 3, 4],
            timestamp: 1234567890,
        };
        
        analyzer.analyze_packet(packet);
        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.get("total_packets"), Some(&1));
        assert_eq!(stats.get("protocol_tcp"), Some(&1));
    }
}