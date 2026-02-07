use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: Instant,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet capture running for {:.2} seconds", duration.as_secs_f64());
        println!("Total packets captured: {}", self.total_packets);
        
        if self.total_packets > 0 {
            let packets_per_second = self.total_packets as f64 / duration.as_secs_f64();
            println!("Packets per second: {:.2}", packets_per_second);
            
            println!("\nProtocol distribution:");
            for (protocol, count) in &self.protocol_counts {
                let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
                println!("  {}: {} ({:.1}%)", protocol, count, percentage);
            }
        }
    }
}

fn handle_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        pnet::packet::ethernet::EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                match ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                        stats.update("TCP");
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            println!(
                                "TCP Packet: {}:{} -> {}:{} [Flags: {:?}]",
                                ipv4_packet.get_source(),
                                tcp_packet.get_source(),
                                ipv4_packet.get_destination(),
                                tcp_packet.get_destination(),
                                tcp_packet.get_flags()
                            );
                        }
                    }
                    IpNextHeaderProtocols::Udp => {
                        stats.update("UDP");
                        println!(
                            "UDP Packet: {} -> {}",
                            ipv4_packet.get_source(),
                            ipv4_packet.get_destination()
                        );
                    }
                    IpNextHeaderProtocols::Icmp => {
                        stats.update("ICMP");
                        println!("ICMP Packet detected");
                    }
                    _ => {
                        stats.update("Other-IPv4");
                    }
                }
            }
        }
        pnet::packet::ethernet::EtherTypes::Ipv6 => {
            stats.update("IPv6");
        }
        pnet::packet::ethernet::EtherTypes::Arp => {
            stats.update("ARP");
            println!("ARP Packet detected");
        }
        _ => {
            stats.update("Other");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.is_up() && !iface.is_loopback() && !iface.ips.is_empty())
        .ok_or("No suitable network interface found")?;

    println!("Starting packet capture on interface: {}", interface.name);

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut last_display = Instant::now();
    let display_interval = Duration::from_secs(5);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_packet(&ethernet_packet, &mut stats);
                }

                if last_display.elapsed() >= display_interval {
                    println!("\n=== Statistics ===");
                    stats.display();
                    println!("==================\n");
                    last_display = Instant::now();
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}