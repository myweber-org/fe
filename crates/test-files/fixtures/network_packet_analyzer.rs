use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::env;
use std::time::Instant;

struct PacketStats {
    total_packets: usize,
    protocol_counts: HashMap<String, usize>,
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
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("\n=== Packet Capture Statistics ===");
        println!("Duration: {:.2} seconds", elapsed);
        println!("Total packets: {}", self.total_packets);
        println!("Packets/sec: {:.2}", self.total_packets as f64 / elapsed);
        
        println!("\nProtocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn handle_transport_packet(packet: &[u8], protocol: &str, stats: &mut PacketStats) {
    match protocol {
        "TCP" => {
            if let Some(tcp_packet) = TcpPacket::new(packet) {
                println!("  TCP: {} -> {} | Seq: {} Ack: {} Win: {}",
                    tcp_packet.get_source(),
                    tcp_packet.get_destination(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_window());
            }
        }
        "UDP" => {
            if let Some(udp_packet) = UdpPacket::new(packet) {
                println!("  UDP: {} -> {} | Length: {}",
                    udp_packet.get_source(),
                    udp_packet.get_destination(),
                    udp_packet.get_length());
            }
        }
        _ => {}
    }
    stats.update(protocol);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let interface_name = if args.len() > 1 {
        &args[1]
    } else {
        println!("Usage: {} <interface_name>", args[0]);
        println!("Available interfaces:");
        for iface in datalink::interfaces() {
            println!("  {}", iface.name);
        }
        return Ok(());
    };

    let interface = datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect("Interface not found");

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop and display statistics\n");

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    match ethernet_packet.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                                println!("[{}] IPv4: {} -> {} | Protocol: {}",
                                    packet_count,
                                    ipv4_packet.get_source(),
                                    ipv4_packet.get_destination(),
                                    ipv4_packet.get_next_level_protocol());
                                
                                match ipv4_packet.get_next_level_protocol() {
                                    IpNextHeaderProtocols::Tcp => {
                                        handle_transport_packet(
                                            ipv4_packet.payload(),
                                            "TCP",
                                            &mut stats
                                        );
                                    }
                                    IpNextHeaderProtocols::Udp => {
                                        handle_transport_packet(
                                            ipv4_packet.payload(),
                                            "UDP",
                                            &mut stats
                                        );
                                    }
                                    IpNextHeaderProtocols::Icmp => {
                                        stats.update("ICMP");
                                        println!("  ICMP packet");
                                    }
                                    _ => {
                                        stats.update("Other");
                                        println!("  Other protocol");
                                    }
                                }
                            }
                        }
                        EtherTypes::Arp => {
                            stats.update("ARP");
                            println!("[{}] ARP packet", packet_count);
                        }
                        _ => {
                            stats.update("Other");
                            println!("[{}] Other Ethernet type", packet_count);
                        }
                    }
                }
                
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

    stats.display();
    Ok(())
}