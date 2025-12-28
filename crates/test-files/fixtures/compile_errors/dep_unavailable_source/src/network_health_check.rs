use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::Packet;
use pnet::transport::{icmp_packet_iter, transport_channel, TransportChannelType::Layer3};
use pnet::transport::TransportProtocol::Ipv4;

const BUFFER_SIZE: usize = 64;

pub fn ping_host(host: IpAddr, timeout_secs: u64) -> Result<bool, String> {
    let protocol = match host {
        IpAddr::V4(_) => Ipv4(Ipv4(1)),
        IpAddr::V6(_) => return Err("IPv6 not supported in this example".to_string()),
    };

    let (mut tx, mut rx) = match transport_channel(BUFFER_SIZE, Layer3(protocol)) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return Err(format!("Failed to create channel: {}", e)),
    };

    let mut icmp_header = [0u8; BUFFER_SIZE];
    let mut icmp_packet = MutableEchoRequestPacket::new(&mut icmp_header).unwrap();
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_sequence_number(1);
    icmp_packet.set_identifier(1234);
    let checksum = pnet::packet::icmp::checksum(&icmp_packet.to_immutable());
    icmp_packet.set_checksum(checksum);

    match tx.send_to(icmp_packet, host) {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to send packet: {}", e)),
    }

    let mut iter = icmp_packet_iter(&mut rx);
    let timeout = Duration::from_secs(timeout_secs);
    match iter.next_with_timeout(timeout) {
        Ok(Some((packet, addr))) => {
            if addr == host && packet.get_icmp_type() == IcmpTypes::EchoReply {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Error receiving packet: {}", e)),
    }
}

pub fn check_network_health(hosts: Vec<IpAddr>) -> Vec<(IpAddr, bool)> {
    let mut results = Vec::new();
    for host in hosts {
        let status = ping_host(host, 2).unwrap_or(false);
        results.push((host, status));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[test]
    fn test_localhost_ping() {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let result = ping_host(localhost, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_health_check_multiple_hosts() {
        let hosts = vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
        ];
        let results = check_network_health(hosts);
        assert_eq!(results.len(), 2);
    }
}