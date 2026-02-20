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

    pub fn get_protocol_name(&self) -> &'static str {
        match self.protocol {
            1 => "ICMP",
            6 => "TCP",
            17 => "UDP",
            _ => "UNKNOWN",
        }
    }

    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }
}

pub struct PacketAnalyzer {
    packet_count: usize,
    protocol_stats: HashMap<u8, usize>,
    source_ip_stats: HashMap<Ipv4Addr, usize>,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            packet_count: 0,
            protocol_stats: HashMap::new(),
            source_ip_stats: HashMap::new(),
        }
    }

    pub fn analyze_packet(&mut self, packet: &NetworkPacket) {
        self.packet_count += 1;

        *self.protocol_stats.entry(packet.protocol).or_insert(0) += 1;
        *self.source_ip_stats.entry(packet.source_ip).or_insert(0) += 1;
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total packets analyzed: {}\n", self.packet_count));
        report.push_str("Protocol statistics:\n");

        for (protocol, count) in &self.protocol_stats {
            let protocol_name = match protocol {
                1 => "ICMP",
                6 => "TCP",
                17 => "UDP",
                _ => "UNKNOWN",
            };
            report.push_str(&format!("  {}: {}\n", protocol_name, count));
        }

        report.push_str("Top source IP addresses:\n");
        let mut sorted_ips: Vec<(&Ipv4Addr, &usize)> = self.source_ip_stats.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));

        for (ip, count) in sorted_ips.iter().take(5) {
            report.push_str(&format!("  {}: {}\n", ip, count));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let packet = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            6,
            vec![1, 2, 3, 4],
            1234567890,
        );

        assert_eq!(packet.get_protocol_name(), "TCP");
        assert_eq!(packet.payload_size(), 4);
    }

    #[test]
    fn test_packet_analyzer() {
        let mut analyzer = PacketAnalyzer::new();

        let packet1 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(192, 168, 1, 2),
            6,
            vec![],
            1234567890,
        );

        let packet2 = NetworkPacket::new(
            Ipv4Addr::new(192, 168, 1, 3),
            Ipv4Addr::new(192, 168, 1, 2),
            17,
            vec![],
            1234567891,
        );

        analyzer.analyze_packet(&packet1);
        analyzer.analyze_packet(&packet2);
        analyzer.analyze_packet(&packet1);

        let report = analyzer.generate_report();
        assert!(report.contains("Total packets analyzed: 3"));
        assert!(report.contains("TCP: 2"));
        assert!(report.contains("UDP: 1"));
    }
}