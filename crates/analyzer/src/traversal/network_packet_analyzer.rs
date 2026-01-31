use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: IpAddr,
    destination_ip: IpAddr,
    protocol: Protocol,
    payload_size: usize,
    timestamp: SystemTime,
    ttl: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl NetworkPacket {
    pub fn new(
        source_ip: IpAddr,
        destination_ip: IpAddr,
        protocol: Protocol,
        payload_size: usize,
        ttl: u8,
    ) -> Self {
        NetworkPacket {
            source_ip,
            destination_ip,
            protocol,
            payload_size,
            timestamp: SystemTime::now(),
            ttl,
        }
    }

    pub fn is_local_traffic(&self) -> bool {
        match (self.source_ip, self.destination_ip) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => {
                src.is_loopback() || dst.is_loopback() || src.is_private() || dst.is_private()
            }
            (IpAddr::V6(src), IpAddr::V6(dst)) => src.is_loopback() || dst.is_loopback(),
            _ => false,
        }
    }

    pub fn is_valid_ttl(&self) -> bool {
        self.ttl > 0 && self.ttl <= 255
    }

    pub fn packet_summary(&self) -> String {
        format!(
            "Packet: {} -> {} | Protocol: {:?} | Size: {} bytes",
            self.source_ip, self.destination_ip, self.protocol, self.payload_size
        )
    }
}

pub struct PacketAnalyzer {
    packets: Vec<NetworkPacket>,
    total_bytes: usize,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packets: Vec::new(),
            total_bytes: 0,
        }
    }

    pub fn add_packet(&mut self, packet: NetworkPacket) {
        self.total_bytes += packet.payload_size;
        self.packets.push(packet);
    }

    pub fn get_protocol_distribution(&self) -> std::collections::HashMap<Protocol, usize> {
        let mut distribution = std::collections::HashMap::new();
        
        for packet in &self.packets {
            *distribution.entry(packet.protocol.clone()).or_insert(0) += 1;
        }
        
        distribution
    }

    pub fn average_packet_size(&self) -> f64 {
        if self.packets.is_empty() {
            0.0
        } else {
            self.total_bytes as f64 / self.packets.len() as f64
        }
    }

    pub fn filter_by_protocol(&self, protocol: Protocol) -> Vec<&NetworkPacket> {
        self.packets
            .iter()
            .filter(|p| p.protocol == protocol)
            .collect()
    }

    pub fn find_largest_packet(&self) -> Option<&NetworkPacket> {
        self.packets.iter().max_by_key(|p| p.payload_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_traffic_detection() {
        let local_packet = NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
            Protocol::TCP,
            1024,
            64,
        );
        
        assert!(local_packet.is_local_traffic());
        
        let public_packet = NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            Protocol::UDP,
            512,
            128,
        );
        
        assert!(!public_packet.is_local_traffic());
    }

    #[test]
    fn test_packet_analyzer_statistics() {
        let mut analyzer = PacketAnalyzer::new();
        
        analyzer.add_packet(NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            Protocol::TCP,
            1500,
            64,
        ));
        
        analyzer.add_packet(NetworkPacket::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            Protocol::UDP,
            512,
            64,
        ));
        
        assert_eq!(analyzer.packets.len(), 2);
        assert_eq!(analyzer.total_bytes, 2012);
        assert_eq!(analyzer.average_packet_size(), 1006.0);
    }
}