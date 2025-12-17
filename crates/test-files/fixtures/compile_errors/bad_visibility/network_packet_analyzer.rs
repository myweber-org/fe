
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args().nth(1).unwrap_or_else(|| "eth0".to_string());
    
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .expect("Interface not found");

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    analyze_packet(&ethernet, packet_count);
                }
                
                if packet_count >= 100 {
                    println!("Captured 100 packets. Stopping.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                continue;
            }
        }
    }

    Ok(())
}

fn analyze_packet(ethernet: &EthernetPacket, count: usize) {
    println!("\nPacket #{}", count);
    println!("  Source MAC: {}", ethernet.get_source());
    println!("  Destination MAC: {}", ethernet.get_destination());
    
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                println!("  IPv4 Packet");
                println!("    Source IP: {}", ipv4.get_source());
                println!("    Destination IP: {}", ipv4.get_destination());
                println!("    Protocol: {}", ipv4.get_next_level_protocol());
                println!("    TTL: {}", ipv4.get_ttl());
                
                match ipv4.get_next_level_protocol() {
                    pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                        if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                            println!("    TCP Packet");
                            println!("      Source Port: {}", tcp.get_source());
                            println!("      Destination Port: {}", tcp.get_destination());
                            println!("      Flags: {:?}", tcp.get_flags());
                            println!("      Window Size: {}", tcp.get_window());
                        }
                    }
                    pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                        if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                            println!("    UDP Packet");
                            println!("      Source Port: {}", udp.get_source());
                            println!("      Destination Port: {}", udp.get_destination());
                            println!("      Length: {}", udp.get_length());
                        }
                    }
                    _ => println!("    Other Protocol"),
                }
            }
        }
        EtherTypes::Ipv6 => {
            println!("  IPv6 Packet (not analyzed in this version)");
        }
        EtherTypes::Arp => {
            println!("  ARP Packet");
        }
        _ => {
            println!("  Other Ethernet Type: {:?}", ethernet.get_ethertype());
        }
    }
    
    println!("  Packet Size: {} bytes", ethernet.packet().len());
}