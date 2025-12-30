use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
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
            eprintln!("Interface {} not found", interface_name);
            std::process::exit(1);
        });

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
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

    println!("Capturing packets on {}...", interface_name);

    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    analyze_packet(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn analyze_packet(ethernet_packet: &EthernetPacket) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                println!(
                    "IPv4 Packet: {} -> {} | Protocol: {}",
                    ipv4_packet.get_source(),
                    ipv4_packet.get_destination(),
                    ipv4_packet.get_next_level_protocol()
                );

                if ipv4_packet.get_next_level_protocol() == pnet::packet::ip::IpNextHeaderProtocols::Tcp {
                    if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                        println!(
                            "  TCP: {} -> {} | Flags: {:?}",
                            tcp_packet.get_source(),
                            tcp_packet.get_destination(),
                            tcp_packet.get_flags()
                        );
                    }
                }
            }
        }
        EtherTypes::Arp => {
            println!("ARP Packet detected");
        }
        _ => {
            println!("Other EtherType: {:?}", ethernet_packet.get_ethertype());
        }
    }
}