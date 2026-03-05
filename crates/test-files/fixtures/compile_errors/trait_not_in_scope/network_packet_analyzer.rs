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
}use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface_name: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface_name)
            .ok_or_else(|| format!("Interface {} not found", interface_name))?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");
        
        let mut packet_counter = 0;
        while let Ok(packet) = self.capture.next_packet() {
            Self::analyze_packet(&packet);
            packet_counter += 1;
            
            if packet_count > 0 && packet_counter >= packet_count {
                break;
            }
        }
        
        println!("Captured {} packets", packet_counter);
        Ok(())
    }

    fn analyze_packet(packet: &pcap::Packet) {
        let header = &packet.header;
        let data = &packet.data;
        
        println!("Packet captured:");
        println!("  Timestamp: {}.{}", header.ts.tv_sec, header.ts.tv_usec);
        println!("  Length: {} bytes", header.len);
        println!("  Captured length: {} bytes", header.caplen);
        
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            match eth_type {
                0x0800 => println!("  Ethernet Type: IPv4"),
                0x0806 => println!("  Ethernet Type: ARP"),
                0x86DD => println!("  Ethernet Type: IPv6"),
                _ => println!("  Ethernet Type: Unknown (0x{:04x})", eth_type),
            }
        }
        
        if data.len() > 0 {
            println!("  First 16 bytes: {:02x?}", &data[..std::cmp::min(16, data.len())]);
        }
        
        println!("---");
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}