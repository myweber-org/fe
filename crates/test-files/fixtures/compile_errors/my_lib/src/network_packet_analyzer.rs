use pcap::{Capture, Device};
use std::error::Error;

pub struct PacketAnalyzer {
    device_name: String,
}

impl PacketAnalyzer {
    pub fn new(device_name: &str) -> Self {
        PacketAnalyzer {
            device_name: device_name.to_string(),
        }
    }

    pub fn capture_packets(&self, count: usize) -> Result<(), Box<dyn Error>> {
        let device = Device::list()?
            .into_iter()
            .find(|dev| dev.name == self.device_name)
            .ok_or_else(|| format!("Device {} not found", self.device_name))?;

        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .open()?;

        println!("Starting packet capture on {}...", self.device_name);
        
        for i in 0..count {
            match cap.next_packet() {
                Ok(packet) => {
                    println!("Packet {}: {} bytes captured", i + 1, packet.header.len);
                    self.analyze_packet(&packet);
                }
                Err(e) => eprintln!("Error capturing packet: {}", e),
            }
        }

        println!("Capture completed.");
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let data = packet.data;
        
        if data.len() >= 14 {
            let dest_mac = &data[0..6];
            let src_mac = &data[6..12];
            let ethertype = u16::from_be_bytes([data[12], data[13]]);
            
            println!("  Source MAC: {:02x?}", src_mac);
            println!("  Destination MAC: {:02x?}", dest_mac);
            println!("  EtherType: 0x{:04x}", ethertype);
            
            match ethertype {
                0x0800 => println!("  Protocol: IPv4"),
                0x0806 => println!("  Protocol: ARP"),
                0x86dd => println!("  Protocol: IPv6"),
                _ => println!("  Protocol: Unknown"),
            }
        }
    }
}

pub fn list_available_devices() -> Result<(), Box<dyn Error>> {
    println!("Available network devices:");
    for device in Device::list()? {
        println!("  - {}", device.name);
        if let Some(desc) = device.desc {
            println!("    Description: {}", desc);
        }
    }
    Ok(())
}