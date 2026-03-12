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
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    source_ips: HashMap<String, u64>,
    destination_ips: HashMap<String, u64>,
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
        println!("\nTop 5 Source IPs:");
        let mut sorted_src: Vec<_> = self.source_ips.iter().collect();
        sorted_src.sort_by(|a, b| b.1.cmp(a.1));
        for (ip, count) in sorted_src.iter().take(5) {
            println!("  {}: {}", ip, count);
        }
    }
}

fn handle_packet(ethernet: &EthernetPacket, stats: &mut PacketStats) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                let src_ip = ipv4_packet.get_source().to_string();
                let dst_ip = ipv4_packet.get_destination().to_string();
                let protocol = match ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                            format!("TCP:{}->{}", tcp_packet.get_source(), tcp_packet.get_destination())
                        } else {
                            "TCP".to_string()
                        }
                    }
                    IpNextHeaderProtocols::Udp => {
                        if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                            format!("UDP:{}->{}", udp_packet.get_source(), udp_packet.get_destination())
                        } else {
                            "UDP".to_string()
                        }
                    }
                    IpNextHeaderProtocols::Icmp => "ICMP".to_string(),
                    _ => "Other".to_string(),
                };
                stats.update(&protocol, &src_ip, &dst_ip);
            }
        }
        EtherTypes::Arp => {
            stats.update("ARP", "N/A", "N/A");
        }
        _ => {
            stats.update("Other", "N/A", "N/A");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <interface_name>", args[0]);
        process::exit(1);
    }

    let interface_name = &args[1];
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == *interface_name)
        .expect("Interface not found");

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            eprintln!("Unsupported channel type");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error creating channel: {}", e);
            process::exit(1);
        }
    };

    println!("Starting packet capture on interface: {}", interface_name);
    println!("Press Ctrl+C to stop and display statistics\n");

    let mut stats = PacketStats::new();
    let mut packet_count = 0;
    let max_packets = 100;

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    handle_packet(&ethernet_packet, &mut stats);
                    packet_count += 1;

                    if packet_count % 10 == 0 {
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }

                    if packet_count >= max_packets {
                        println!("\n\nReached maximum packet limit ({}).", max_packets);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }

    stats.display_summary();
}