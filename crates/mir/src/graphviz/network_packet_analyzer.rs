
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub source_ip: IpAddr,
    pub destination_ip: IpAddr,
    pub protocol: Protocol,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}

impl From<u8> for Protocol {
    fn from(value: u8) -> Self {
        match value {
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            1 => Protocol::ICMP,
            _ => Protocol::Other(value),
        }
    }
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<Protocol, usize>,
    ip_traffic: HashMap<IpAddr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;
        
        *self.protocol_stats
            .entry(packet.protocol.clone())
            .or_insert(0) += 1;
        
        *self.ip_traffic
            .entry(packet.source_ip)
            .or_insert(0) += 1;
        
        *self.ip_traffic
            .entry(packet.destination_ip)
            .or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> PacketStatistics {
        PacketStatistics {
            total_packets: self.packet_count,
            top_protocol: self.find_top_protocol(),
            unique_ips: self.ip_traffic.len(),
        }
    }

    fn find_top_protocol(&self) -> Option<(Protocol, usize)> {
        self.protocol_stats
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(proto, &count)| (proto.clone(), count))
    }

    pub fn parse_raw_packet(raw_data: &[u8]) -> Option<NetworkPacket> {
        if raw_data.len() < 20 {
            return None;
        }

        let version = (raw_data[0] >> 4) & 0x0F;
        if version != 4 {
            return None;
        }

        let protocol_byte = raw_data[9];
        let source_ip = IpAddr::V4(Ipv4Addr::new(
            raw_data[12],
            raw_data[13],
            raw_data[14],
            raw_data[15],
        ));
        
        let destination_ip = IpAddr::V4(Ipv4Addr::new(
            raw_data[16],
            raw_data[17],
            raw_data[18],
            raw_data[19],
        ));

        let ihl = (raw_data[0] & 0x0F) as usize * 4;
        let payload = if raw_data.len() > ihl {
            raw_data[ihl..].to_vec()
        } else {
            Vec::new()
        };

        Some(NetworkPacket {
            source_ip,
            destination_ip,
            protocol: Protocol::from(protocol_byte),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}

#[derive(Debug)]
pub struct PacketStatistics {
    pub total_packets: usize,
    pub top_protocol: Option<(Protocol, usize)>,
    pub unique_ips: usize,
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
    fn test_packet_parsing() {
        let mut raw_packet = vec![0x45, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x40, 0x00, 
                                  0x40, 0x06, 0x00, 0x00, 0xC0, 0xA8, 0x01, 0x01,
                                  0xC0, 0xA8, 0x01, 0x64];
        
        raw_packet.extend(vec![0x00, 0x01, 0x02, 0x03]);

        let packet = PacketAnalyzer::parse_raw_packet(&raw_packet).unwrap();
        
        assert_eq!(packet.source_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(packet.destination_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
        assert_eq!(packet.protocol, Protocol::TCP);
    }

    #[test]
    fn test_analyzer_statistics() {
        let mut analyzer = PacketAnalyzer::new();
        
        let packet1 = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            protocol: Protocol::TCP,
            payload: vec![1, 2, 3],
            timestamp: 1234567890,
        };

        let packet2 = NetworkPacket {
            source_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            destination_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)),
            protocol: Protocol::UDP,
            payload: vec![4, 5, 6],
            timestamp: 1234567891,
        };

        analyzer.analyze_packet(&packet1);
        analyzer.analyze_packet(&packet2);
        analyzer.analyze_packet(&packet1);

        let stats = analyzer.get_statistics();
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.unique_ips, 3);
        
        if let Some((proto, count)) = stats.top_protocol {
            assert_eq!(proto, Protocol::TCP);
            assert_eq!(count, 2);
        }
    }
}