
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Unknown(u8),
}

#[derive(Debug)]
pub struct PacketHeader {
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: Protocol,
    pub length: usize,
    pub timestamp: u64,
}

pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_stats: HashMap<Protocol, u64>,
    ip_traffic: HashMap<String, u64>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, header: PacketHeader) {
        self.packet_count += 1;
        
        *self.protocol_stats.entry(header.protocol.clone()).or_insert(0) += 1;
        
        *self.ip_traffic.entry(header.source_ip.clone()).or_insert(0) += 1;
        *self.ip_traffic.entry(header.destination_ip.clone()).or_insert(0) += 1;
        
        self.print_packet_info(&header);
    }

    fn print_packet_info(&self, header: &PacketHeader) {
        println!("Packet #{}", self.packet_count);
        println!("  Source: {}", header.source_ip);
        println!("  Destination: {}", header.destination_ip);
        println!("  Protocol: {:?}", header.protocol);
        println!("  Length: {} bytes", header.length);
        println!("  Timestamp: {}", header.timestamp);
        println!("---");
    }

    pub fn get_statistics(&self) -> String {
        let mut stats = String::new();
        stats.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        stats.push_str("Protocol distribution:\n");
        
        for (protocol, count) in &self.protocol_stats {
            let percentage = (*count as f64 / self.packet_count as f64) * 100.0;
            stats.push_str(&format!("  {:?}: {} ({:.2}%)\n", protocol, count, percentage));
        }
        
        stats
    }

    pub fn detect_protocol(protocol_number: u8) -> Protocol {
        match protocol_number {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Unknown(protocol_number),
        }
    }

    pub fn parse_ipv4_address(bytes: [u8; 4]) -> String {
        Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string()
    }

    pub fn parse_ipv6_address(bytes: [u8; 16]) -> String {
        Ipv6Addr::from(bytes).to_string()
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
    fn test_ipv4_parsing() {
        let ip_bytes = [192, 168, 1, 1];
        let ip_string = PacketAnalyzer::parse_ipv4_address(ip_bytes);
        assert_eq!(ip_string, "192.168.1.1");
    }

    #[test]
    fn test_packet_analysis() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet = PacketHeader {
            source_ip: "192.168.1.100".to_string(),
            destination_ip: "8.8.8.8".to_string(),
            protocol: Protocol::TCP,
            length: 1500,
            timestamp: 1234567890,
        };
        
        analyzer.analyze_packet(packet);
        assert_eq!(analyzer.packet_count, 1);
        
        let stats = analyzer.get_statistics();
        assert!(stats.contains("Total packets analyzed: 1"));
        assert!(stats.contains("TCP"));
    }
}