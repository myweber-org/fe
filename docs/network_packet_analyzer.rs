use pnet::datalink::{self, Channel, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
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
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unsupported channel type"),
        Err(e) => panic!("Failed to create channel: {}", e),
    };

    println!("Starting packet capture on interface: {}", interface.name);
    let mut packet_count = 0;

    loop {
        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    analyze_ethernet_frame(&ethernet);
                }
                
                if packet_count >= 100 {
                    println!("Captured {} packets. Stopping.", packet_count);
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

fn analyze_ethernet_frame(ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                analyze_ipv4_packet(&ipv4);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
        }
        EtherTypes::Arp => {
            println!("ARP packet detected");
        }
        _ => {
            println!("Other Ethernet type: {:?}", ethernet.get_ethertype());
        }
    }
}

fn analyze_ipv4_packet(ipv4: &Ipv4Packet) {
    let source = ipv4.get_source();
    let destination = ipv4.get_destination();
    let protocol = ipv4.get_next_level_protocol();
    
    match protocol {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                analyze_tcp_packet(&tcp, source, destination);
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            println!("UDP packet: {} -> {}", source, destination);
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            println!("ICMP packet: {} -> {}", source, destination);
        }
        _ => {
            println!("Other IP protocol: {:?} from {} to {}", protocol, source, destination);
        }
    }
}

fn analyze_tcp_packet(tcp: &TcpPacket, source: std::net::Ipv4Addr, destination: std::net::Ipv4Addr) {
    let src_port = tcp.get_source();
    let dst_port = tcp.get_destination();
    let flags = tcp.get_flags();
    
    let flag_str = format!(
        "{}{}{}{}{}{}",
        if flags & 0x20 != 0 { "U" } else { "" },
        if flags & 0x10 != 0 { "A" } else { "" },
        if flags & 0x08 != 0 { "P" } else { "" },
        if flags & 0x04 != 0 { "R" } else { "" },
        if flags & 0x02 != 0 { "S" } else { "" },
        if flags & 0x01 != 0 { "F" } else { "" },
    );
    
    println!(
        "TCP: {}:{} -> {}:{} [Flags: {}]",
        source, src_port, destination, dst_port, flag_str
    );
}