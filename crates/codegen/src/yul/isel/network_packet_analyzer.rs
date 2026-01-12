rust
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
    
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    process_ethernet_frame(&ethernet_packet);
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
            println!("Unknown ethertype: {:?}", ethernet.get_ethertype());
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    match ipv4.get_next_level_protocol() {
        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4.payload()) {
                println!(
                    "TCP Packet: {}:{} -> {}:{} | Seq: {} Ack: {} Window: {}",
                    ipv4.get_source(),
                    tcp_packet.get_source(),
                    ipv4.get_destination(),
                    tcp_packet.get_destination(),
                    tcp_packet.get_sequence(),
                    tcp_packet.get_acknowledgement(),
                    tcp_packet.get_window()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4.payload()) {
                println!(
                    "UDP Packet: {}:{} -> {}:{} | Length: {}",
                    ipv4.get_source(),
                    udp_packet.get_source(),
                    ipv4.get_destination(),
                    udp_packet.get_destination(),
                    udp_packet.get_length()
                );
            }
        }
        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
            println!("ICMP packet from {} to {}", ipv4.get_source(), ipv4.get_destination());
        }
        _ => {
            println!(
                "Other IP protocol: {} from {} to {}",
                ipv4.get_next_level_protocol(),
                ipv4.get_source(),
                ipv4.get_destination()
            );
        }
    }
}
```