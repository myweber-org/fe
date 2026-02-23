use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
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
                    analyze_packet(&eth_packet, packet_count);
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

fn analyze_packet(eth_packet: &EthernetPacket, count: usize) {
    println!("\n=== Packet #{} ===", count);
    println!("Source MAC: {}", eth_packet.get_source());
    println!("Destination MAC: {}", eth_packet.get_destination());
    
    match eth_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
                analyze_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            if let Some(ipv6_packet) = Ipv6Packet::new(eth_packet.payload()) {
                analyze_ipv6_packet(&ipv6_packet);
            }
        }
        EtherTypes::Arp => {
            println!("Protocol: ARP");
        }
        _ => {
            println!("Protocol: Other (0x{:04x})", eth_packet.get_ethertype());
        }
    }
}

fn analyze_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    println!("Protocol: IPv4");
    println!("Source IP: {}", ipv4_packet.get_source());
    println!("Destination IP: {}", ipv4_packet.get_destination());
    println!("TTL: {}", ipv4_packet.get_ttl());
    
    match ipv4_packet.get_next_level_protocol() {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                analyze_tcp_packet(&tcp_packet);
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                analyze_udp_packet(&udp_packet);
            }
        }
        _ => {
            println!("Transport: Other ({})", ipv4_packet.get_next_level_protocol());
        }
    }
}

fn analyze_ipv6_packet(ipv6_packet: &Ipv6Packet) {
    println!("Protocol: IPv6");
    println!("Source IP: {}", ipv6_packet.get_source());
    println!("Destination IP: {}", ipv6_packet.get_destination());
    println!("Hop Limit: {}", ipv6_packet.get_hop_limit());
}

fn analyze_tcp_packet(tcp_packet: &TcpPacket) {
    println!("Transport: TCP");
    println!("Source Port: {}", tcp_packet.get_source());
    println!("Destination Port: {}", tcp_packet.get_destination());
    println!("Sequence: {}", tcp_packet.get_sequence());
    println!("Acknowledgment: {}", tcp_packet.get_acknowledgement());
    
    let flags = tcp_packet.get_flags();
    let flag_str = format!(
        "{}{}{}{}{}{}",
        if flags & 0x20 != 0 { "U" } else { "" },
        if flags & 0x10 != 0 { "A" } else { "" },
        if flags & 0x08 != 0 { "P" } else { "" },
        if flags & 0x04 != 0 { "R" } else { "" },
        if flags & 0x02 != 0 { "S" } else { "" },
        if flags & 0x01 != 0 { "F" } else { "" },
    );
    
    println!("Flags: {} (0x{:02x})", flag_str, flags);
    println!("Window Size: {}", tcp_packet.get_window());
}

fn analyze_udp_packet(udp_packet: &UdpPacket) {
    println!("Transport: UDP");
    println!("Source Port: {}", udp_packet.get_source());
    println!("Destination Port: {}", udp_packet.get_destination());
    println!("Length: {}", udp_packet.get_length());
}