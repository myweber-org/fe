extern crate pnet;

use pnet::datalink::{self, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
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

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
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
                    process_ethernet_packet(&ethernet_packet);
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}

fn process_ethernet_packet(ethernet_packet: &EthernetPacket) {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                process_ipv4_packet(&ipv4_packet);
            }
        }
        EtherTypes::Ipv6 => {
            println!("IPv6 packet detected");
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
    match ipv4_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                println!(
                    "TCP Packet: {}:{} -> {}:{}",
                    ipv4_packet.get_source(),
                    tcp_packet.get_source(),
                    ipv4_packet.get_destination(),
                    tcp_packet.get_destination()
                );
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                println!(
                    "UDP Packet: {}:{} -> {}:{}",
                    ipv4_packet.get_source(),
                    udp_packet.get_source(),
                    ipv4_packet.get_destination(),
                    udp_packet.get_destination()
                );
            }
        }
        IpNextHeaderProtocols::Icmp => {
            println!("ICMP Packet from {}", ipv4_packet.get_source());
        }
        _ => {
            println!(
                "Other IP protocol: {:?}",
                ipv4_packet.get_next_level_protocol()
            );
        }
    }
}