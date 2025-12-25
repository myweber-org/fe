
use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    capture: Capture<pcap::Active>,
}

impl PacketAnalyzer {
    pub fn new(interface: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == interface)
            .ok_or_else(|| format!("Interface {} not found", interface))?;

        let capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        Ok(PacketAnalyzer { capture })
    }

    pub fn start_capture(&mut self, packet_count: i32) -> Result<(), Box<dyn Error>> {
        println!("Starting packet capture on interface...");

        for _ in 0..packet_count {
            match self.capture.next_packet() {
                Ok(packet) => {
                    self.analyze_packet(&packet);
                }
                Err(e) => {
                    eprintln!("Error capturing packet: {}", e);
                    break;
                }
            }
        }

        println!("Packet capture completed.");
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let header = packet.header;
        let data = packet.data;

        println!("Packet captured:");
        println!("  Timestamp: {}.{}", header.ts.tv_sec, header.ts.tv_usec);
        println!("  Length: {} bytes", header.len);
        println!("  Captured length: {} bytes", header.caplen);
        
        if data.len() >= 14 {
            let eth_type = u16::from_be_bytes([data[12], data[13]]);
            println!("  Ethernet Type: 0x{:04x}", eth_type);
            
            match eth_type {
                0x0800 => println!("    IPv4 Packet"),
                0x0806 => println!("    ARP Packet"),
                0x86DD => println!("    IPv6 Packet"),
                _ => println!("    Unknown Protocol"),
            }
        }
        
        if data.len() > 0 {
            println!("  First 16 bytes: {:02x?}", &data[..std::cmp::min(16, data.len())]);
        }
        
        println!();
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  {}", device.name);
        if let Some(desc) = device.desc {
            println!("    Description: {}", desc);
        }
    }
    Ok(())
}