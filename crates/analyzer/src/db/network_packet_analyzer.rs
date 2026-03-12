
use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
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
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface_name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                let ethernet = EthernetPacket::new(packet).unwrap();
                
                match ethernet.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                            match ipv4_packet.get_next_level_protocol() {
                                IpNextHeaderProtocols::Tcp => {
                                    if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                        println!(
                                            "Packet #{}: TCP {}:{} -> {}:{} [Flags: {:?}]",
                                            packet_count,
                                            ipv4_packet.get_source(),
                                            tcp_packet.get_source(),
                                            ipv4_packet.get_destination(),
                                            tcp_packet.get_destination(),
                                            tcp_packet.get_flags()
                                        );
                                    }
                                }
                                _ => {
                                    println!(
                                        "Packet #{}: IPv4 {} -> {} Protocol: {}",
                                        packet_count,
                                        ipv4_packet.get_source(),
                                        ipv4_packet.get_destination(),
                                        ipv4_packet.get_next_level_protocol()
                                    );
                                }
                            }
                        }
                    }
                    EtherTypes::Ipv6 => {
                        println!("Packet #{}: IPv6 packet detected", packet_count);
                    }
                    EtherTypes::Arp => {
                        println!("Packet #{}: ARP packet detected", packet_count);
                    }
                    _ => {
                        println!("Packet #{}: Other Ethernet type: {:?}", packet_count, ethernet.get_ethertype());
                    }
                }

                if packet_count >= 100 {
                    println!("Captured 100 packets. Stopping.");
                    break;
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