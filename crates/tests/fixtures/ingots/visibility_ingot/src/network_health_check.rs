use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use rand::Rng;
use tokio::net::UdpSocket;
use tokio::time::sleep;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const TEST_COUNT: u32 = 10;

#[derive(Debug)]
pub struct NetworkMetrics {
    pub avg_latency_ms: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
    pub healthy: bool,
}

pub async fn check_network_health(target: IpAddr, port: u16) -> NetworkMetrics {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let target_addr = SocketAddr::new(target, port);
    
    let mut latencies = Vec::new();
    let mut lost_packets = 0;
    
    for seq in 0..TEST_COUNT {
        let payload = generate_payload(seq);
        let start = Instant::now();
        
        match tokio::time::timeout(
            Duration::from_millis(TIMEOUT_MS),
            socket.send_to(&payload, &target_addr)
        ).await {
            Ok(Ok(_)) => {
                latencies.push(start.elapsed().as_millis() as f64);
            }
            _ => {
                lost_packets += 1;
            }
        }
        
        if seq < TEST_COUNT - 1 {
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    calculate_metrics(latencies, lost_packets)
}

fn generate_payload(sequence: u32) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut payload = vec![0u8; PACKET_SIZE];
    
    payload[0..4].copy_from_slice(&sequence.to_be_bytes());
    rng.fill(&mut payload[4..]);
    
    payload
}

fn calculate_metrics(latencies: Vec<f64>, lost_packets: u32) -> NetworkMetrics {
    if latencies.is_empty() {
        return NetworkMetrics {
            avg_latency_ms: 0.0,
            packet_loss_percent: 100.0,
            jitter_ms: 0.0,
            healthy: false,
        };
    }
    
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let packet_loss = (lost_packets as f64 / TEST_COUNT as f64) * 100.0;
    
    let mut jitter_sum = 0.0;
    for i in 1..latencies.len() {
        jitter_sum += (latencies[i] - latencies[i-1]).abs();
    }
    let jitter = jitter_sum / (latencies.len() - 1) as f64;
    
    let healthy = avg_latency < 50.0 && packet_loss < 5.0 && jitter < 20.0;
    
    NetworkMetrics {
        avg_latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        jitter_ms: jitter,
        healthy,
    }
}

#[tokio::main]
async fn main() {
    let target = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let metrics = check_network_health(target, 53).await;
    
    println!("Network Health Check Results:");
    println!("Average Latency: {:.2} ms", metrics.avg_latency_ms);
    println!("Packet Loss: {:.1}%", metrics.packet_loss_percent);
    println!("Jitter: {:.2} ms", metrics.jitter_ms);
    println!("Healthy: {}", metrics.healthy);
}