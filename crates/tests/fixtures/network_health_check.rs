use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::thread;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const MAX_RETRIES: u8 = 3;

#[derive(Debug)]
pub struct NetworkMetrics {
    pub latency_ms: Option<u64>,
    pub packet_loss: f32,
    pub reachable: bool,
}

pub fn check_network_health(target: IpAddr, port: u16) -> NetworkMetrics {
    let socket_addr = SocketAddr::new(target, port);
    let mut successful_pings = 0;
    let mut total_latency = Duration::new(0, 0);
    
    for attempt in 0..MAX_RETRIES {
        if let Some(latency) = send_ping(&socket_addr) {
            successful_pings += 1;
            total_latency += latency;
            thread::sleep(Duration::from_millis(100));
        } else if attempt < MAX_RETRIES - 1 {
            thread::sleep(Duration::from_millis(200));
        }
    }
    
    let packet_loss = 1.0 - (successful_pings as f32 / MAX_RETRIES as f32);
    let avg_latency = if successful_pings > 0 {
        Some((total_latency.as_millis() / successful_pings as u128) as u64)
    } else {
        None
    };
    
    NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss,
        reachable: successful_pings > 0,
    }
}

fn send_ping(target: &SocketAddr) -> Option<Duration> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS))).ok()?;
    
    let payload = [0u8; PACKET_SIZE];
    let start = Instant::now();
    
    if socket.send_to(&payload, target).is_ok() {
        let mut buffer = [0u8; PACKET_SIZE];
        if socket.recv_from(&mut buffer).is_ok() {
            return Some(start.elapsed());
        }
    }
    
    None
}

pub fn monitor_network(targets: Vec<IpAddr>, interval_secs: u64) {
    let port = 33434;
    
    loop {
        println!("=== Network Health Check ===");
        
        for target in &targets {
            let metrics = check_network_health(*target, port);
            println!("Target: {} - Latency: {:?}ms, Packet Loss: {:.1}%, Reachable: {}",
                     target,
                     metrics.latency_ms,
                     metrics.packet_loss * 100.0,
                     metrics.reachable);
        }
        
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_localhost_check() {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let metrics = check_network_health(localhost, 8080);
        
        assert!(!metrics.reachable, "Localhost port 8080 should not be reachable");
        assert_eq!(metrics.successful_pings, 0);
    }
}