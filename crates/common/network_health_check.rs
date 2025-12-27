use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;
use std::thread;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);
const PING_COUNT: usize = 3;

pub struct NetworkProbe {
    target: IpAddr,
    ports: Vec<u16>,
}

impl NetworkProbe {
    pub fn new(target: IpAddr, ports: Vec<u16>) -> Self {
        NetworkProbe { target, ports }
    }

    pub fn check_connectivity(&self) -> ProbeResult {
        let ping_success = self.perform_ping_check();
        let port_results = self.scan_ports();
        
        ProbeResult {
            target: self.target,
            ping_available: ping_success,
            open_ports: port_results,
        }
    }

    fn perform_ping_check(&self) -> bool {
        match self.target {
            IpAddr::V4(ipv4) => self.ping_ipv4(ipv4),
            IpAddr::V6(_) => false, // IPv6 ping not implemented
        }
    }

    fn ping_ipv4(&self, target: Ipv4Addr) -> bool {
        let mut successes = 0;
        
        for _ in 0..PING_COUNT {
            if self.send_icmp_echo(target) {
                successes += 1;
            }
            thread::sleep(Duration::from_millis(500));
        }
        
        successes > 0
    }

    fn send_icmp_echo(&self, target: Ipv4Addr) -> bool {
        // Simplified ICMP echo simulation
        // In production, use proper ICMP library like `ping`
        let socket_addr = SocketAddr::new(IpAddr::V4(target), 0);
        
        match TcpStream::connect_timeout(&socket_addr, DEFAULT_TIMEOUT) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn scan_ports(&self) -> Vec<PortStatus> {
        let mut results = Vec::new();
        
        for &port in &self.ports {
            let status = self.check_port(port);
            results.push(PortStatus { port, open: status });
        }
        
        results
    }

    fn check_port(&self, port: u16) -> bool {
        let socket_addr = SocketAddr::new(self.target, port);
        
        match TcpStream::connect_timeout(&socket_addr, DEFAULT_TIMEOUT) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

pub struct ProbeResult {
    pub target: IpAddr,
    pub ping_available: bool,
    pub open_ports: Vec<PortStatus>,
}

pub struct PortStatus {
    pub port: u16,
    pub open: bool,
}

pub fn analyze_network_health(target: IpAddr, ports: Vec<u16>) -> String {
    let probe = NetworkProbe::new(target, ports);
    let result = probe.check_connectivity();
    
    format!(
        "Target: {}\nPing: {}\nOpen ports: {}",
        result.target,
        if result.ping_available { "OK" } else { "FAILED" },
        result.open_ports.iter()
            .filter(|p| p.open)
            .map(|p| p.port.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_localhost_connectivity() {
        let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let ports = vec![80, 443, 8080];
        
        let result = analyze_network_health(target, ports);
        assert!(result.contains("Target: 127.0.0.1"));
    }
}