
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
}use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PacketStats {
    pub total_packets: usize,
    pub protocol_counts: HashMap<String, usize>,
    pub start_time: Instant,
}

impl PacketStats {
    pub fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    pub fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    pub fn display_summary(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet capture summary:");
        println!("  Duration: {:.2} seconds", duration.as_secs_f64());
        println!("  Total packets: {}", self.total_packets);
        println!("  Packets/second: {:.2}", 
                 self.total_packets as f64 / duration.as_secs_f64());
        
        println!("  Protocol distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("    {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

pub fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    let timeout = Duration::from_secs(duration_secs);
    let start_time = Instant::now();

    println!("Starting packet capture on {} for {} seconds...", interface_name, duration_secs);
    
    while start_time.elapsed() < timeout {
        match rx.next() {
            Ok(packet) => {
                if let Some(eth_packet) = EthernetPacket::new(packet) {
                    process_ethernet_packet(&eth_packet, &mut stats);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    Ok(stats)
}

fn process_ethernet_packet(eth_packet: &EthernetPacket, stats: &mut PacketStats) {
    match eth_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ip_packet) = Ipv4Packet::new(eth_packet.payload()) {
                process_ipv4_packet(&ip_packet, stats);
            }
        }
        EtherTypes::Ipv6 => {
            stats.update("IPv6");
        }
        EtherTypes::Arp => {
            stats.update("ARP");
        }
        _ => {
            stats.update("Other");
        }
    }
}

fn process_ipv4_packet(ip_packet: &Ipv4Packet, stats: &mut PacketStats) {
    match ip_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                stats.update(&format!("TCP {}->{}", src_port, dst_port));
            } else {
                stats.update("TCP");
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
                let src_port = udp_packet.get_source();
                let dst_port = udp_packet.get_destination();
                stats.update(&format!("UDP {}->{}", src_port, dst_port));
            } else {
                stats.update("UDP");
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.update("ICMP");
        }
        _ => {
            stats.update("Other-IPv4");
        }
    }
}

pub fn list_interfaces() -> Vec<String> {
    datalink::interfaces()
        .iter()
        .map(|iface| iface.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats_new() {
        let stats = PacketStats::new();
        assert_eq!(stats.total_packets, 0);
        assert!(stats.protocol_counts.is_empty());
    }

    #[test]
    fn test_packet_stats_update() {
        let mut stats = PacketStats::new();
        stats.update("TCP");
        stats.update("UDP");
        stats.update("TCP");
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.protocol_counts.get("TCP"), Some(&2));
        assert_eq!(stats.protocol_counts.get("UDP"), Some(&1));
    }

    #[test]
    fn test_list_interfaces() {
        let interfaces = list_interfaces();
        assert!(!interfaces.is_empty());
    }
}