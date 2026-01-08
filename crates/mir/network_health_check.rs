use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use pnet::packet::icmp::{echo_request, IcmpTypes};
use pnet::packet::Packet;
use pnet::transport::{icmp_packet_iter, transport_channel, TransportChannelType::Layer3};
use pnet::transport::TransportProtocol::Ipv4;

const ICMP_BUFFER_SIZE: usize = 4096;

pub fn ping_host(destination: Ipv4Addr, timeout_secs: u64) -> Result<bool, String> {
    let protocol = Layer3(Ipv4(Ipv4(Ipv4)));
    let (mut tx, mut rx) = match transport_channel(ICMP_BUFFER_SIZE, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return Err(format!("Failed to create transport channel: {}", e)),
    };

    let mut icmp_header = echo_request::MutableEchoRequestPacket::new(vec![0u8; echo_request::MutableEchoRequestPacket::minimum_packet_size()]).unwrap();
    icmp_header.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_header.set_identifier(rand::random::<u16>());
    icmp_header.set_sequence_number(1);
    let checksum = pnet::packet::icmp::checksum(&icmp_header.to_immutable());
    icmp_header.set_checksum(checksum);

    let destination_ip = IpAddr::V4(destination);
    match tx.send_to(icmp_header.packet(), destination_ip) {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to send ICMP packet: {}", e)),
    }

    let mut iter = icmp_packet_iter(&mut rx);
    let timeout = Duration::from_secs(timeout_secs);
    match iter.next_with_timeout(timeout) {
        Ok(Some((packet, addr))) => {
            if addr == destination_ip && packet.get_icmp_type() == IcmpTypes::EchoReply {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Error receiving packet: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_localhost() {
        let result = ping_host(Ipv4Addr::new(127, 0, 0, 1), 2);
        assert!(result.is_ok());
    }
}