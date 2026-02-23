use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct TrafficStats {
    total_packets: u64,
    total_bytes: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: Instant,
}

impl TrafficStats {
    fn new() -> Self {
        TrafficStats {
            total_packets: 0,
            total_bytes: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, protocol: &str, packet_len: usize) {
        self.total_packets += 1;
        self.total_bytes += packet_len as u64;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display_summary(&self) {
        let duration = self.start_time.elapsed();
        println!("Traffic Analysis Summary:");
        println!("  Duration: {:.2?}", duration);
        println!("  Total Packets: {}", self.total_packets);
        println!("  Total Bytes: {}", self.total_bytes);
        
        if duration > Duration::from_secs(0) {
            let packets_per_sec = self.total_packets as f64 / duration.as_secs_f64();
            let bytes_per_sec = self.total_bytes as f64 / duration.as_secs_f64();
            println!("  Packets/sec: {:.2}", packets_per_sec);
            println!("  Bytes/sec: {:.2}", bytes_per_sec);
        }

        println!("  Protocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("    {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn process_packet(ethernet: &EthernetPacket, stats: &mut TrafficStats) {
    let packet_len = ethernet.packet().len();
    
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                match ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                        stats.update("TCP", packet_len);
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            let src_port = tcp_packet.get_source();
                            let dst_port = tcp_packet.get_destination();
                            println!("TCP Packet: {}:{} -> {}:{} ({} bytes)", 
                                   ipv4_packet.get_source(), src_port,
                                   ipv4_packet.get_destination(), dst_port,
                                   packet_len);
                        }
                    }
                    IpNextHeaderProtocols::Udp => {
                        stats.update("UDP", packet_len);
                        if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                            let src_port = udp_packet.get_source();
                            let dst_port = udp_packet.get_destination();
                            println!("UDP Packet: {}:{} -> {}:{} ({} bytes)", 
                                   ipv4_packet.get_source(), src_port,
                                   ipv4_packet.get_destination(), dst_port,
                                   packet_len);
                        }
                    }
                    IpNextHeaderProtocols::Icmp => {
                        stats.update("ICMP", packet_len);
                        println!("ICMP Packet: {} -> {} ({} bytes)", 
                               ipv4_packet.get_source(),
                               ipv4_packet.get_destination(),
                               packet_len);
                    }
                    _ => {
                        stats.update("Other-IPv4", packet_len);
                        println!("Other IPv4 Protocol: {} -> {} ({} bytes)", 
                               ipv4_packet.get_source(),
                               ipv4_packet.get_destination(),
                               packet_len);
                    }
                }
            }
        }
        EtherTypes::Arp => {
            stats.update("ARP", packet_len);
            println!("ARP Packet ({} bytes)", packet_len);
        }
        _ => {
            stats.update("Other-Ethernet", packet_len);
            println!("Other Ethernet Type ({} bytes)", packet_len);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting network packet analyzer...");
    
    let interfaces = datalink::interfaces();
    let default_interface = interfaces
        .iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .ok_or("No suitable network interface found")?;
    
    println!("Using interface: {}", default_interface.name);
    
    let (mut tx, mut rx) = match datalink::channel(&default_interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = TrafficStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    println!("Capturing up to {} packets...", max_packets);
    println!("Press Ctrl+C to stop early\n");

    while packet_count < max_packets {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_packet(&ethernet_packet, &mut stats);
                    packet_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    println!("\nCapture complete!");
    stats.display_summary();
    
    Ok(())
}