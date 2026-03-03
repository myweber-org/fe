
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
}extern crate pnet;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn main() {
    let interface_name = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: {} <interface>", env::args().next().unwrap());
        std::process::exit(1);
    });

    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .unwrap_or_else(|| {
            eprintln!("No such interface: {}", interface_name);
            std::process::exit(1);
        });

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to create channel: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting packet capture on {}...", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                let ethernet = EthernetPacket::new(packet).unwrap();
                process_ethernet_frame(&ethernet);
                
                if packet_count % 100 == 0 {
                    println!("Captured {} packets...", packet_count);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_frame(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected (not processed)");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Unknown ethertype: {:?}", ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                println!(
                    "TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                    ipv4.get_source(),
                    tcp.get_source(),
                    ipv4.get_destination(),
                    tcp.get_destination(),
                    tcp.get_flags()
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                println!(
                    "UDP Packet: {}:{} -> {}:{}",
                    ipv4.get_source(),
                    udp.get_source(),
                    ipv4.get_destination(),
                    udp.get_destination()
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("ICMP packet from {} to {}", ipv4.get_source(), ipv4.get_destination());
        }
        _ => {
            println!(
                "Other IPv4 protocol: {} from {} to {}",
                ipv4.get_next_level_protocol(),
                ipv4.get_source(),
                ipv4.get_destination()
            );
        }
    }
}