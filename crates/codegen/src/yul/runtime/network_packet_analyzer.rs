use pnet::datalink::{self, Channel, Config};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
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
        println!("Packet Capture Summary:");
        println!("  Duration: {:.2} seconds", duration.as_secs_f64());
        println!("  Total Packets: {}", self.total_packets);
        println!("  Packets/second: {:.2}", 
                 self.total_packets as f64 / duration.as_secs_f64());
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

pub fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    let config = Config::default();
    let (mut rx, _) = match datalink::channel(&interface, config) {
        Ok(Channel::Ethernet(tx, rx)) => (rx, tx),
        Ok(_) => return Err("Unsupported channel type".to_string()),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut stats = PacketStats::new();
    let timeout = Duration::from_secs(duration_secs);
    let start_time = Instant::now();

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Capture will run for {} seconds", duration_secs);

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
                stats.update("TCP");
                
                if src_port == 80 || dst_port == 80 {
                    stats.update("HTTP");
                } else if src_port == 443 || dst_port == 443 {
                    stats.update("HTTPS");
                } else if src_port == 22 || dst_port == 22 {
                    stats.update("SSH");
                }
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
                let src_port = udp_packet.get_source();
                let dst_port = udp_packet.get_destination();
                stats.update("UDP");
                
                if src_port == 53 || dst_port == 53 {
                    stats.update("DNS");
                } else if src_port == 67 || dst_port == 67 || src_port == 68 || dst_port == 68 {
                    stats.update("DHCP");
                }
            }
        }
        IpNextHeaderProtocols::Icmp => {
            stats.update("ICMP");
        }
        _ => {
            stats.update("Other-IP");
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
        stats.update("TCP");
        stats.update("UDP");
        
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