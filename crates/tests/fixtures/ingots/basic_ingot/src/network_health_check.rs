
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;

const PACKET_COUNT: usize = 10;
const TIMEOUT_SECONDS: u64 = 2;

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub avg_latency_ms: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
    pub target: IpAddr,
}

impl NetworkMetrics {
    pub fn new(target: IpAddr) -> Self {
        NetworkMetrics {
            avg_latency_ms: 0.0,
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
            target,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.packet_loss_percent < 20.0 && self.avg_latency_ms < 100.0
    }
}

pub struct NetworkProbe {
    target: SocketAddr,
    packet_size: usize,
}

impl NetworkProbe {
    pub fn new(ip: IpAddr, port: u16, packet_size: usize) -> Self {
        NetworkProbe {
            target: SocketAddr::new(ip, port),
            packet_size,
        }
    }

    pub fn measure(&self) -> NetworkMetrics {
        let mut latencies = Vec::with_capacity(PACKET_COUNT);
        let mut successful_packets = 0;

        for _ in 0..PACKET_COUNT {
            if let Some(latency) = self.send_probe() {
                latencies.push(latency);
                successful_packets += 1;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        self.calculate_metrics(latencies, successful_packets)
    }

    fn send_probe(&self) -> Option<Duration> {
        let start = Instant::now();
        
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECONDS))).ok()?;
        
        let packet = self.generate_packet();
        socket.send_to(&packet, self.target).ok()?;
        
        let mut buffer = vec![0u8; self.packet_size];
        socket.recv_from(&mut buffer).ok()?;
        
        Some(start.elapsed())
    }

    fn generate_packet(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..self.packet_size).map(|_| rng.gen()).collect()
    }

    fn calculate_metrics(&self, latencies: Vec<Duration>, successful: usize) -> NetworkMetrics {
        let packet_loss = ((PACKET_COUNT - successful) as f64 / PACKET_COUNT as f64) * 100.0;
        
        if latencies.is_empty() {
            return NetworkMetrics::new(self.target.ip());
        }

        let total_latency: Duration = latencies.iter().sum();
        let avg_latency = total_latency.as_secs_f64() / latencies.len() as f64 * 1000.0;

        let mut jitter_sum = 0.0;
        for i in 1..latencies.len() {
            let diff = (latencies[i].as_secs_f64() - latencies[i-1].as_secs_f64()).abs();
            jitter_sum += diff;
        }
        let jitter = if latencies.len() > 1 {
            jitter_sum / (latencies.len() - 1) as f64 * 1000.0
        } else {
            0.0
        };

        NetworkMetrics {
            avg_latency_ms: avg_latency,
            packet_loss_percent: packet_loss,
            jitter_ms: jitter,
            target: self.target.ip(),
        }
    }
}

pub fn check_network_health(target: &str, port: u16) -> Option<NetworkMetrics> {
    let ip_addr: IpAddr = target.parse().ok()?;
    let probe = NetworkProbe::new(ip_addr, port, 64);
    Some(probe.measure())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_healthy() {
        let metrics = NetworkMetrics {
            avg_latency_ms: 50.0,
            packet_loss_percent: 10.0,
            jitter_ms: 5.0,
            target: IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        };
        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_metrics_unhealthy() {
        let metrics = NetworkMetrics {
            avg_latency_ms: 200.0,
            packet_loss_percent: 30.0,
            jitter_ms: 50.0,
            target: IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        };
        assert!(!metrics.is_healthy());
    }
}