use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};

struct ServerStatus {
    address: SocketAddr,
    is_online: bool,
    latency: Option<Duration>,
}

fn check_server(address: SocketAddr, timeout: Duration) -> (bool, Option<Duration>) {
    let start = std::time::Instant::now();
    match TcpStream::connect_timeout(&address, timeout) {
        Ok(_) => {
            let elapsed = start.elapsed();
            (true, Some(elapsed))
        }
        Err(_) => (false, None),
    }
}

fn main() {
    let servers = vec![
        "8.8.8.8:53".parse().unwrap(),
        "1.1.1.1:53".parse().unwrap(),
        "127.0.0.1:80".parse().unwrap(),
    ];

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for server in servers {
        let results_clone = Arc::clone(&results);
        let handle = thread::spawn(move || {
            let (is_online, latency) = check_server(server, Duration::from_secs(2));
            let status = ServerStatus {
                address: server,
                is_online,
                latency,
            };
            results_clone.lock().unwrap().push(status);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_results = results.lock().unwrap();
    for status in final_results.iter() {
        if status.is_online {
            if let Some(latency) = status.latency {
                println!("Server {} is online. Latency: {:?}", status.address, latency);
            }
        } else {
            println!("Server {} is offline.", status.address);
        }
    }
}