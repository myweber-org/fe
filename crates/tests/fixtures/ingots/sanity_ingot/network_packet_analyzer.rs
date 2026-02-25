use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
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
                    process_ethernet_frame(&ethernet);
                }
                if packet_count >= 100 {
                    println!("Captured {} packets. Stopping.", packet_count);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn process_ethernet_frame(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
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
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    let source = ipv4.get_source();
    let destination = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();
    let ttl = ipv4.get_ttl();

    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                println!(
                    "TCP Packet: {}:{} -> {}:{} | TTL: {}",
                    source,
                    tcp.get_source(),
                    destination,
                    tcp.get_destination(),
                    ttl
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                println!(
                    "UDP Packet: {}:{} -> {}:{} | TTL: {}",
                    source,
                    udp.get_source(),
                    destination,
                    udp.get_destination(),
                    ttl
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("ICMP Packet: {} -> {} | TTL: {}", source, destination, ttl);
        }
        _ => {
            println!(
                "Other IP protocol {:?}: {} -> {} | TTL: {}",
                protocol, source, destination, ttl
            );
        }
    }
}use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
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
                if let Some(eth_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&eth_packet, packet_count);
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet: {}", e);
                break;
            }
        }

        if packet_count >= 100 {
            println!("Captured {} packets. Stopping.", packet_count);
            break;
        }
    }

    Ok(())
}

fn process_ethernet_frame(ethernet: &EthernetPacket, count: usize) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet.payload()) {
                process_ipv4_packet(&ipv4_packet, count);
            }
        }
        EtherTypes::Ipv6 => {
            println!("Packet {}: IPv6 packet detected", count);
        }
        EtherTypes::Arp => {
            println!("Packet {}: ARP packet detected", count);
        }
        _ => {
            println!("Packet {}: Unknown ethertype: {:?}", count, ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet, count: usize) {
    let source = ipv4.get_source();
    let destination = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();

    match protocol {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                let src_port = tcp_packet.get_source();
                let dst_port = tcp_packet.get_destination();
                println!(
                    "Packet {}: TCP {}:{} -> {}:{} ({} bytes)",
                    count,
                    source,
                    src_port,
                    destination,
                    dst_port,
                    tcp_packet.packet().len()
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                let src_port = udp_packet.get_source();
                let dst_port = udp_packet.get_destination();
                println!(
                    "Packet {}: UDP {}:{} -> {}:{} ({} bytes)",
                    count,
                    source,
                    src_port,
                    destination,
                    dst_port,
                    udp_packet.packet().len()
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("Packet {}: ICMP {} -> {}", count, source, destination);
        }
        _ => {
            println!(
                "Packet {}: Protocol {:?} {} -> {}",
                count, protocol, source, destination
            );
        }
    }
}