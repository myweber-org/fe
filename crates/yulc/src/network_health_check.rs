use std::process::Command;
use std::time::Duration;
use std::thread;

fn main() {
    let targets = vec![
        ("8.8.8.8", "Google DNS"),
        ("1.1.1.1", "Cloudflare DNS"),
        ("example.com", "Example Website"),
    ];

    for (target, description) in targets {
        println!("Checking connectivity to {} ({})...", description, target);
        
        if ping_target(target) {
            println!("✓ {} is reachable via ICMP", description);
        } else {
            println!("✗ {} is not reachable via ICMP", description);
        }

        if target.contains('.') && !target.contains("://") {
            let http_target = format!("http://{}", target);
            if check_http(&http_target) {
                println!("✓ {} responds to HTTP", description);
            } else {
                println!("✗ {} does not respond to HTTP", description);
            }
        }

        println!();
        thread::sleep(Duration::from_secs(1));
    }
}

fn ping_target(host: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(&["-n", "1", "-w", "1000", host])
            .output()
    } else {
        Command::new("ping")
            .args(&["-c", "1", "-W", "1", host])
            .output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn check_http(url: &str) -> bool {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    match client {
        Ok(client) => match client.head(url).send() {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}