use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface_name: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface_name)
            .ok_or("Interface not found")?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");
        
        let mut count = 0;
        while let Ok(packet) = self.capture.next_packet() {
            println!("Packet {} captured:", count + 1);
            println!("  Timestamp: {:?}", packet.header.ts);
            println!("  Length: {} bytes", packet.header.len);
            println!("  Captured length: {} bytes", packet.header.caplen);
            
            self.analyze_packet(&packet);
            
            count += 1;
            if count >= packet_count {
                break;
            }
        }
        
        println!("Captured {} packets", count);
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        if packet.data.len() >= 14 {
            let eth_type = u16::from_be_bytes([packet.data[12], packet.data[13]]);
            match eth_type {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86DD => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown (0x{:04x})", eth_type),
            }
            
            if eth_type == 0x0800 && packet.data.len() >= 34 {
                let protocol = packet.data[23];
                match protocol {
                    1 => println!("  IP Protocol: ICMP"),
                    6 => println!("  IP Protocol: TCP"),
                    17 => println!("  IP Protocol: UDP"),
                    _ => println!("  IP Protocol: {}", protocol),
                }
                
                let src_ip = format!("{}.{}.{}.{}", 
                    packet.data[26], packet.data[27], 
                    packet.data[28], packet.data[29]);
                let dst_ip = format!("{}.{}.{}.{}", 
                    packet.data[30], packet.data[31], 
                    packet.data[32], packet.data[33]);
                println!("  Source IP: {}", src_ip);
                println!("  Destination IP: {}", dst_ip);
            }
        }
        
        println!("  First 16 bytes: {:02x?}", &packet.data[..std::cmp::min(16, packet.data.len())]);
        println!();
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}: {}", device.name, device.desc.unwrap_or_default());
    }
    Ok(())
}