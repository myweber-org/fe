
use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use pnet::transport::{transport_channel, TransportChannelType::Layer4};
use pnet::transport::TransportProtocol::{Ipv4, Ipv6};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::Packet;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const BUFFER_SIZE: usize = 1024;

pub enum Protocol {
    TCP,
    ICMP,
}

pub struct HealthCheckResult {
    pub is_successful: bool,
    pub latency: Option<Duration>,
    pub error: Option<String>,
}

pub struct NetworkChecker {
    timeout: Duration,
}

impl NetworkChecker {
    pub fn new(timeout: Duration) -> Self {
        NetworkChecker { timeout }
    }

    pub fn default() -> Self {
        NetworkChecker::new(DEFAULT_TIMEOUT)
    }

    pub fn check_tcp(&self, host: &str, port: u16) -> HealthCheckResult {
        let addr_str = format!("{}:{}", host, port);
        
        match addr_str.parse::<SocketAddr>() {
            Ok(addr) => {
                let start = Instant::now();
                match TcpStream::connect_timeout(&addr, self.timeout) {
                    Ok(_) => {
                        let latency = start.elapsed();
                        HealthCheckResult {
                            is_successful: true,
                            latency: Some(latency),
                            error: None,
                        }
                    }
                    Err(e) => HealthCheckResult {
                        is_successful: false,
                        latency: None,
                        error: Some(format!("TCP connection failed: {}", e)),
                    },
                }
            }
            Err(e) => HealthCheckResult {
                is_successful: false,
                latency: None,
                error: Some(format!("Invalid address: {}", e)),
            },
        }
    }

    pub fn check_icmp(&self, host: &str) -> HealthCheckResult {
        let start = Instant::now();
        
        let (mut tx, mut rx) = match transport_channel(BUFFER_SIZE, Layer4(Ipv4(IpNextHeaderProtocols::Icmp))) {
            Ok((tx, rx)) => (tx, rx),
            Err(e) => {
                return HealthCheckResult {
                    is_successful: false,
                    latency: None,
                    error: Some(format!("Failed to create channel: {}", e)),
                }
            }
        };

        let icmp_packet = self.build_icmp_echo_request();
        
        match tx.send_to(icmp_packet, host.parse().unwrap()) {
            Ok(_) => {
                let mut buffer = [0u8; BUFFER_SIZE];
                match rx.recv_from() {
                    Ok((packet, _)) => {
                        let latency = start.elapsed();
                        if let Some(icmp_packet) = IcmpPacket::new(&packet) {
                            if icmp_packet.get_icmp_type() == IcmpTypes::EchoReply {
                                HealthCheckResult {
                                    is_successful: true,
                                    latency: Some(latency),
                                    error: None,
                                }
                            } else {
                                HealthCheckResult {
                                    is_successful: false,
                                    latency: None,
                                    error: Some("Received non-echo-reply ICMP packet".to_string()),
                                }
                            }
                        } else {
                            HealthCheckResult {
                                is_successful: false,
                                latency: None,
                                error: Some("Failed to parse ICMP packet".to_string()),
                            }
                        }
                    }
                    Err(e) => HealthCheckResult {
                        is_successful: false,
                        latency: None,
                        error: Some(format!("Failed to receive response: {}", e)),
                    },
                }
            }
            Err(e) => HealthCheckResult {
                is_successful: false,
                latency: None,
                error: Some(format!("Failed to send ICMP packet: {}", e)),
            },
        }
    }

    fn build_icmp_echo_request(&self) -> Vec<u8> {
        let mut buffer = vec![0u8; 8];
        buffer[0] = 8;
        buffer[1] = 0;
        let checksum = self.calculate_checksum(&buffer);
        buffer[2] = (checksum >> 8) as u8;
        buffer[3] = (checksum & 0xFF) as u8;
        buffer
    }

    fn calculate_checksum(&self, data: &[u8]) -> u16 {
        let mut sum = 0u32;
        
        for chunk in data.chunks(2) {
            let word = if chunk.len() == 2 {
                ((chunk[0] as u32) << 8) | (chunk[1] as u32)
            } else {
                (chunk[0] as u32) << 8
            };
            sum += word;
        }
        
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }

    pub fn perform_check(&self, protocol: Protocol, target: &str, port: Option<u16>) -> HealthCheckResult {
        match protocol {
            Protocol::TCP => {
                if let Some(p) = port {
                    self.check_tcp(target, p)
                } else {
                    HealthCheckResult {
                        is_successful: false,
                        latency: None,
                        error: Some("Port required for TCP check".to_string()),
                    }
                }
            }
            Protocol::ICMP => self.check_icmp(target),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_check_success() {
        let checker = NetworkChecker::default();
        let result = checker.check_tcp("google.com", 80);
        assert!(result.is_successful);
        assert!(result.latency.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tcp_check_failure() {
        let checker = NetworkChecker::new(Duration::from_millis(100));
        let result = checker.check_tcp("nonexistent.local", 9999);
        assert!(!result.is_successful);
        assert!(result.error.is_some());
    }
}