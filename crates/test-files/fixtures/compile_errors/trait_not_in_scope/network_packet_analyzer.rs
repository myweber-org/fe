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

    println!("Starting packet capture on interface: {}", interface_name);
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_frame(ethernet_packet: &EthernetPacket) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected (not processed)");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet_packet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    println!(
        "IPv4 Packet: {} -> {} | Protocol: {} | Length: {}",
        ipv4_packet.get_source(),
        ipv4_packet.get_destination(),
        ipv4_packet.get_next_level_protocol(),
        ipv4_packet.get_total_length()
    );

    match ipv4_packet.get_next_level_protocol() {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                process_tcp_packet(&tcp_packet);
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            println!("UDP packet detected");
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            println!("ICMP packet detected");
        }
        _ => {
            println!("Other IP protocol: {:?}", ipv4_packet.get_next_level_protocol());
        }
    }
}

fn process_tcp_packet(tcp_packet: &TcpPacket) {
    println!(
        "TCP Segment: {} -> {} | Flags: {:?} | Window: {}",
        tcp_packet.get_source(),
        tcp_packet.get_destination(),
        tcp_packet.get_flags(),
        tcp_packet.get_window()
    );
}