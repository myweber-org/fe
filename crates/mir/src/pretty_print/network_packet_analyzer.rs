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

        println!("Capture completed.");
        Ok(())
    }

    fn analyze_packet(&self, packet: &pcap::Packet) {
        let header = packet.header;
        let data = packet.data;

        println!(
            "Packet received: {} bytes, timestamp: {}.{}",
            header.len,
            header.ts.tv_sec,
            header.ts.tv_usec
        );

        if data.len() >= 14 {
            let dest_mac = &data[0..6];
            let src_mac = &data[6..12];
            let ethertype = u16::from_be_bytes([data[12], data[13]]);

            println!(
                "Ethernet Frame: SRC {:02X?}, DST {:02X?}, Type: 0x{:04X}",
                src_mac, dest_mac, ethertype
            );

            match ethertype {
                0x0800 => println!("Protocol: IPv4"),
                0x0806 => println!("Protocol: ARP"),
                0x86DD => println!("Protocol: IPv6"),
                _ => println!("Protocol: Unknown"),
            }
        }

        if data.len() > 34 {
            let protocol = data[23];
            let src_ip = format!("{}.{}.{}.{}", data[26], data[27], data[28], data[29]);
            let dst_ip = format!("{}.{}.{}.{}", data[30], data[31], data[32], data[33]);

            println!("IP Header: SRC {}, DST {}", src_ip, dst_ip);

            match protocol {
                6 => println!("Transport: TCP"),
                17 => println!("Transport: UDP"),
                1 => println!("Transport: ICMP"),
                _ => println!("Transport: Protocol {}", protocol),
            }
        }

        println!("{}", "-".repeat(50));
    }
}

pub fn list_interfaces() -> Result<(), Box<dyn Error>> {
    println!("Available network interfaces:");
    for device in Device::list()? {
        println!("  - {}", device.name);
        if let Some(desc) = device.desc {
            println!("    Description: {}", desc);
        }
    }
    Ok(())
}