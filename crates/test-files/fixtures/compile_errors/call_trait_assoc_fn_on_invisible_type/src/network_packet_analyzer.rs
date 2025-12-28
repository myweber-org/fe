use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::process;

struct PacketStats {
    total_packets: usize,
    protocol_counts: HashMap<String, usize>,
    source_ips: HashMap<String, usize>,
    destination_ips: HashMap<String, usize>,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ips: HashMap::new(),
        }
    }

    fn update(&mut self, protocol: &str, src_ip: &str, dst_ip: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
        *self.source_ips.entry(src_ip.to_string()).or_insert(0) += 1;
        *self.destination_ips.entry(dst_ip.to_string()).or_insert(0) += 1;
    }

    fn display_summary(&self) {
        println!("Packet Capture Summary:");
        println!("Total packets captured: {}", self.total_packets);
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            println!("  {}: {}", protocol, count);
        }
        println!("\nTop Source IPs:");
        let mut sorted_src: Vec<_> = self.source_ips.iter().collect();
        sorted_src.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_src.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
        println!("\nTop Destination IPs:");
        let mut sorted_dst: Vec<_> = self.destination_ips.iter().collect();
        sorted_dst.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_dst.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn handle_transport_protocol(
    source: &str,
    destination: &str,
    protocol: u8,
    payload: &[u8],
    stats: &mut PacketStats,
) {
    match protocol {
        6 => {
            if let Some(tcp_packet) = TcpPacket::new(payload) {
                stats.update("TCP", source, destination);
                println!(
                    "TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                    source,
                    tcp_packet.get_source(),
                    destination,
                    tcp_packet.get_destination(),
                    tcp_packet.get_flags()
                );
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(payload) {
                stats.update("UDP", source, destination);
                println!(
                    "UDP Packet: {}:{} -> {}:{}",
                    source,
                    udp_packet.get_source(),
                    destination,
                    udp_packet.get_destination()
                );
            }
        }
        _ => {
            stats.update("Other", source, destination);
            println!("Other Protocol {}: {} -> {}", protocol, source, destination);
        }
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        let source = ipv4_packet.get_source().to_string();
        let destination = ipv4_packet.get_destination().to_string();
        let protocol = ipv4_packet.get_next_level_protocol();

        match protocol {
            IpNextHeaderProtocols::Tcp => {
                handle_transport_protocol(&source, &destination, 6, ipv4_packet.payload(), stats);
            }
            IpNextHeaderProtocols::Udp => {
                handle_transport_protocol(&source, &destination, 17, ipv4_packet.payload(), stats);
            }
            _ => {
                stats.update("IPv4-Other", &source, &destination);
                println!("IPv4 Packet: {} -> {} [Protocol: {}]", source, destination, protocol);
            }
        }
    }
}

fn capture_packets(interface_name: &str, packet_limit: usize) -> Result<(), String> {
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
    let mut packet_count = 0;

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture and display statistics\n");

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            handle_ipv4_packet(&ethernet_packet, &mut stats);
                        }
                        EtherTypes::Ipv6 => {
                            stats.update("IPv6", "N/A", "N/A");
                            println!("IPv6 Packet detected");
                        }
                        EtherTypes::Arp => {
                            stats.update("ARP", "N/A", "N/A");
                            println!("ARP Packet detected");
                        }
                        _ => {
                            stats.update("Other-EtherType", "N/A", "N/A");
                        }
                    }
                }

                packet_count += 1;
                if packet_count >= packet_limit {
                    println!("\nReached packet limit of {}", packet_limit);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display_summary();
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <interface> <packet_limit>", args[0]);
        eprintln!("Example: {} eth0 100", args[0]);
        process::exit(1);
    }

    let interface = &args[1];
    let packet_limit: usize = match args[2].parse() {
        Ok(limit) => limit,
        Err(_) => {
            eprintln!("Invalid packet limit. Please provide a positive integer.");
            process::exit(1);
        }
    };

    if let Err(e) = capture_packets(interface, packet_limit) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
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
        println!("Packet capture summary (over {} seconds):", duration.as_secs());
        println!("Total packets captured: {}", self.total_packets);
        
        if duration.as_secs() > 0 {
            let packets_per_second = self.total_packets as f64 / duration.as_secs() as f64;
            println!("Average packets/second: {:.2}", packets_per_second);
        }

        println!("\nProtocol distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} packets ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn handle_transport_packet(packet: &[u8], stats: &mut PacketStats) {
    if let Some(tcp_packet) = TcpPacket::new(packet) {
        stats.update("TCP");
        println!("TCP Packet: {} -> {}, Seq: {}, Ack: {}",
                 tcp_packet.get_source(),
                 tcp_packet.get_destination(),
                 tcp_packet.get_sequence(),
                 tcp_packet.get_acknowledgement());
    } else if let Some(udp_packet) = UdpPacket::new(packet) {
        stats.update("UDP");
        println!("UDP Packet: {} -> {}, Length: {}",
                 udp_packet.get_source(),
                 udp_packet.get_destination(),
                 udp_packet.get_length());
    } else {
        stats.update("Other Transport");
    }
}

fn handle_ipv4_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        println!("IPv4 Packet: {} -> {}, Protocol: {}, TTL: {}",
                 ipv4_packet.get_source(),
                 ipv4_packet.get_destination(),
                 ipv4_packet.get_next_level_protocol(),
                 ipv4_packet.get_ttl());

        match ipv4_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => handle_transport_packet(ipv4_packet.payload(), stats),
            IpNextHeaderProtocols::Udp => handle_transport_packet(ipv4_packet.payload(), stats),
            _ => stats.update("Other IPv4"),
        }
    }
}

pub fn start_capture(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
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

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Capture will run for {} seconds", duration_secs);

    while start_time.elapsed() < timeout {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    stats.total_packets += 1;

                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            stats.update("IPv4");
                            handle_ipv4_packet(&ethernet_packet, &mut stats);
                        }
                        EtherTypes::Ipv6 => {
                            stats.update("IPv6");
                        }
                        EtherTypes::Arp => {
                            stats.update("ARP");
                        }
                        _ => {
                            stats.update("Other Ethernet");
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats() {
        let mut stats = PacketStats::new();
        assert_eq!(stats.total_packets, 0);
        
        stats.update("TCP");
        stats.update("UDP");
        stats.update("TCP");
        
        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.protocol_counts.get("TCP"), Some(&2));
        assert_eq!(stats.protocol_counts.get("UDP"), Some(&1));
    }
}