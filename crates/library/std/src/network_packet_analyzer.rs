use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkPacket {
    source_ip: Ipv4Addr,
    destination_ip: Ipv4Addr,
    protocol: u8,
    payload: Vec<u8>,
    timestamp: u64,
}

#[derive(Debug)]
pub struct PacketAnalyzer {
    packet_count: u64,
    protocol_distribution: HashMap<u8, u64>,
    ip_traffic: HashMap<Ipv4Addr, u64>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_distribution: HashMap::new(),
            ip_traffic: HashMap::new(),
        }
    }

    pub fn process_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;

        *self.protocol_distribution
            .entry(packet.protocol)
            .or_insert(0) += 1;

        *self.ip_traffic
            .entry(packet.source_ip)
            .or_insert(0) += 1;
        *self.ip_traffic
            .entry(packet.destination_ip)
            .or_insert(0) += 1;
    }

    pub fn get_statistics(&self) -> AnalyzerStats {
        let top_source = self.find_top_ip();
        let most_common_protocol = self.find_most_common_protocol();

        AnalyzerStats {
            total_packets: self.packet_count,
            unique_ips: self.ip_traffic.len(),
            top_source_ip: top_source,
            most_common_protocol,
        }
    }

    fn find_top_ip(&self) -> Option<(Ipv4Addr, u64)> {
        self.ip_traffic
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&ip, &count)| (ip, count))
    }

    fn find_most_common_protocol(&self) -> Option<(u8, u64)> {
        self.protocol_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&protocol, &count)| (protocol, count))
    }
}

#[derive(Debug)]
pub struct AnalyzerStats {
    pub total_packets: u64,
    pub unique_ips: usize,
    pub top_source_ip: Option<(Ipv4Addr, u64)>,
    pub most_common_protocol: Option<(u8, u64)>,
}

impl NetworkPacket {
    pub fn new(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
        protocol: u8,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        NetworkPacket {
            source_ip,
            destination_ip,
            protocol,
            payload,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();

        let packet1 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            6,
            vec![1, 2, 3, 4],
            1234567890,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 3),
            17,
            vec![5, 6, 7, 8],
            1234567891,
        );

        analyzer.process_packet(&packet1);
        analyzer.process_packet(&packet2);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.unique_ips, 3);
        assert_eq!(stats.top_source_ip.unwrap().0, Ipv4Addr::new(192, 168, 1, 1));
    }
}