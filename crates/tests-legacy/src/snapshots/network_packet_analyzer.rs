use pcap::{Capture, Device};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    ARP,
    Other(u16),
}

impl Protocol {
    fn from_ethertype(ethertype: u16) -> Self {
        match ethertype {
            0x0800 => Protocol::IPv4,
            0x0806 => Protocol::ARP,
            0x86DD => Protocol::IPv6,
            _ => Protocol::Other(ethertype),
        }
    }
}

pub struct PacketStats {
    pub total_packets: usize,
    pub protocol_counts: HashMap<Protocol, usize>,
    pub byte_count: usize,
}

impl PacketStats {
    pub fn new() -> Self {
        Self {
            total_packets: 0,
            protocol_counts: HashMap::new(),
            byte_count: 0,
        }
    }

    pub fn add_packet(&mut self, protocol: Protocol, length: usize) {
        self.total_packets += 1;
        self.byte_count += length;
        *self.protocol_counts.entry(protocol).or_insert(0) += 1;
    }

    pub fn print_summary(&self) {
        println!("Packet Capture Summary:");
        println!("Total Packets: {}", self.total_packets);
        println!("Total Bytes: {}", self.byte_count);
        println!("Protocol Distribution:");
        
        for (protocol, count) in &self.protocol_counts {
            let percentage = (*count as f64 / self.total_packets as f64) * 100.0;
            println!("  {:?}: {} ({:.2}%)", protocol, count, percentage);
        }
    }
}

pub fn capture_packets(device_name: &str, count: usize) -> Result<PacketStats, pcap::Error> {
    let device = Device::list()?
        .into_iter()
        .find(|d| d.name == device_name)
        .ok_or_else(|| pcap::Error::InvalidString)?;

    let mut cap = Capture::from_device(device)?
        .promisc(true)
        .snaplen(65535)
        .timeout(1000)
        .open()?;

    let mut stats = PacketStats::new();

    for _ in 0..count {
        if let Ok(packet) = cap.next_packet() {
            let ethertype = u16::from_be_bytes([packet.data[12], packet.data[13]]);
            let protocol = Protocol::from_ethertype(ethertype);
            stats.add_packet(protocol, packet.data.len());
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_stats() {
        let mut stats = PacketStats::new();
        stats.add_packet(Protocol::TCP, 1500);
        stats.add_packet(Protocol::UDP, 512);
        stats.add_packet(Protocol::TCP, 1024);

        assert_eq!(stats.total_packets, 3);
        assert_eq!(stats.byte_count, 3036);
        assert_eq!(stats.protocol_counts.get(&Protocol::TCP), Some(&2));
        assert_eq!(stats.protocol_counts.get(&Protocol::UDP), Some(&1));
    }
}