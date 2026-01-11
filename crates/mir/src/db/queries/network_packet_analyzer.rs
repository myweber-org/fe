rust
use pnet::datalink::{self, Channel, DataLinkReceiver, NetworkInterface};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::Packet;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PacketStats {
    total_packets: u64,
    protocol_counts: HashMap<String, u64>,
    start_time: Instant,
}

impl PacketStats {
    fn new() -> Self {
        PacketStats {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn update(&mut self, protocol: &str) {
        self.total_packets += 1;
        *self.protocol_counts.entry(protocol.to_string()).or_insert(0) += 1;
    }

    fn display_summary(&self) {
        let duration = self.start_time.elapsed();
        println!("Packet Capture Summary:");
        println!("  Duration: {:.2?}", duration);
        println!("  Total Packets: {}", self.total_packets);
        
        if duration > Duration::from_secs(0) {
            let packets_per_second = self.total_packets as f64 / duration.as_secs_f64();
            println!("  Packets/sec: {:.2}", packets_per_second);
        }
        
        println!("  Protocol Distribution:");
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("    {}: {} ({:.1}%)", protocol, count, percentage);
        }
    }
}

fn capture_packets(interface_name: &str, max_packets: u64) -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == interface_name)
        .ok_or_else(|| format!("Interface {} not found", interface_name))?;

    println!("Starting capture on interface: {}", interface.name);
    println!("MAC: {}", interface.mac);
    if let Some(ip) = interface.ips.first() {
        println!("IP: {}", ip);
    }

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err("Unsupported channel type".into()),
        Err(e) => return Err(format!("Failed to create channel: {}", e).into()),
    };

    let mut stats = PacketStats::new();
    let mut packet_count = 0;

    println!("Capturing up to {} packets...", max_packets);
    println!("Press Ctrl+C to stop early\n");

    loop {
        if packet_count >= max_packets {
            println!("Reached maximum packet count of {}", max_packets);
            break;
        }

        match rx.next() {
            Ok(packet) => {
                packet_count += 1;
                process_packet(&packet, &mut stats);
                
                if packet_count % 100 == 0 {
                    print!("\rPackets captured: {}", packet_count);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                continue;
            }
        }
    }

    println!("\n\nCapture complete!");
    stats.display_summary();
    Ok(())
}

fn process_packet(ethernet_data: &[u8], stats: &mut PacketStats) {
    if let Some(ethernet_packet) = EthernetPacket::new(ethernet_data) {
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                stats.update("IPv4");
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    match ipv4_packet.get_next_level_protocol() {
                        pnet::packet::ip::IpNextHeaderProtocols::Tcp => {
                            stats.update("TCP");
                            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                let src_port = tcp_packet.get_source();
                                let dst_port = tcp_packet.get_destination();
                                let flags = tcp_packet.get_flags();
                                
                                if packet_count <= 10 {
                                    println!(
                                        "TCP: {}:{} -> {}:{} | Flags: {:b} | Seq: {}",
                                        ipv4_packet.get_source(),
                                        src_port,
                                        ipv4_packet.get_destination(),
                                        dst_port,
                                        flags,
                                        tcp_packet.get_sequence()
                                    );
                                }
                            }
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Udp => {
                            stats.update("UDP");
                        }
                        pnet::packet::ip::IpNextHeaderProtocols::Icmp => {
                            stats.update("ICMP");
                        }
                        _ => {
                            stats.update("Other-IPv4");
                        }
                    }
                }
            }
            EtherTypes::Ipv6 => {
                stats.update("IPv6");
            }
            EtherTypes::Arp => {
                stats.update("ARP");
            }
            _ => {
                stats.update("Other");
            }
        }
    }
}

fn main() {
    let interface_name = "eth0";
    let max_packets = 1000;

    if let Err(e) = capture_packets(interface_name, max_packets) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```