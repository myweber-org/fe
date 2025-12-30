
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::process;

struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ports: HashMap<u16, u64>,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            source_ips: HashMap::new(),
            destination_ports: HashMap::new(),
        }
    }

    fn increment_protocol(&mut self, protocol: &str) {
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn increment_source_ip(&mut self, ip: &str) {
        *self.source_ips.entry(ip.to_string()).or_insert(0) += 1;
    }

    fn increment_destination_port(&mut self, port: u16) {
        *self.destination_ports.entry(port).or_insert(0) += 1;
    }

    fn display_summary(&self, duration_secs: u64) {
        println!("\n=== Packet Capture Summary ===");
        println!("Capture Duration: {} seconds", duration_secs);
        println!("Total Packets: {}", self.total_packets);
        println!("Packets/sec: {:.2}", self.total_packets as f64 / duration_secs as f64);
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.2}%)", protocol, count, percentage);
        }
        
        println!("\nTop 5 Source IPs:");
        let mut sorted_ips: Vec<_> = self.source_ips.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_ips.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
        
        println!("\nTop 5 Destination Ports:");
        let mut sorted_ports: Vec<_> = self.destination_ports.iter().collect();
        sorted_ports.sort_by(|a, b| b.1.cmp(a.1));
        for (port, count) in sorted_ports.iter().take(5) {
            println!("  {}: {}", port, count);
        }
    }
}

fn handle_transport_protocol(
    source_ip: &str,
    protocol: u8,
    packet: &[u8],
    stats: &mut PacketStats,
) {
    match protocol {
        6 => {
            if let Some(tcp_packet) = TcpPacket::new(packet) {
                stats.increment_destination_port(tcp_packet.get_destination());
                stats.increment_protocol("TCP");
            }
        }
        17 => {
            if let Some(udp_packet) = UdpPacket::new(packet) {
                stats.increment_destination_port(udp_packet.get_destination());
                stats.increment_protocol("UDP");
            }
        }
        1 => {
            stats.increment_protocol("ICMP");
        }
        _ => {
            stats.increment_protocol(&format!("Protocol-{}", protocol));
        }
    }
    stats.increment_source_ip(source_ip);
}

fn handle_ipv4_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
        let source_ip = ipv4_packet.get_source().to_string();
        let protocol = ipv4_packet.get_next_level_protocol();
        
        handle_transport_protocol(&source_ip, protocol.0, ipv4_packet.payload(), stats);
    }
}

fn handle_ipv6_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    if let Some(ipv6_packet) = Ipv6Packet::new(ethernet.payload()) {
        let source_ip = ipv6_packet.get_source().to_string();
        let protocol = ipv6_packet.get_next_header();
        
        handle_transport_protocol(&source_ip, protocol.0, ipv6_packet.payload(), stats);
    }
}

fn capture_packets(interface_name: &str, duration_secs: u64) -> Result<PacketStats, String> {
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
    let start_time = std::time::Instant::now();

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop capture...");

    while start_time.elapsed().as_secs() < duration_secs {
        match rx.next() {
            Ok(packet) => {
                stats.total_packets += 1;
                
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => handle_ipv4_packet(&ethernet_packet, &mut stats),
                        EtherTypes::Ipv6 => handle_ipv6_packet(&ethernet_packet, &mut stats),
                        EtherTypes::Arp => stats.increment_protocol("ARP"),
                        _ => stats.increment_protocol("Other"),
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }

    Ok(stats)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <interface> <duration_seconds>", args[0]);
        eprintln!("Example: {} eth0 10", args[0]);
        process::exit(1);
    }

    let interface = &args[1];
    let duration: u64 = match args[2].parse() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Error: Duration must be a positive integer");
            process::exit(1);
        }
    };

    if duration == 0 {
        eprintln!("Error: Duration must be greater than 0");
        process::exit(1);
    }

    match capture_packets(interface, duration) {
        Ok(stats) => stats.display_summary(duration),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}