use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink;

pub fn list_network_interfaces() -> Vec<(String, Vec<IpAddr>)> {
    let interfaces = pnet_datalink::interfaces();
    let mut result = Vec::new();

    for interface in interfaces {
        if !interface.is_up() || interface.is_loopback() {
            continue;
        }

        let mut ips = Vec::new();
        for ip_network in interface.ips {
            match ip_network.ip() {
                IpAddr::V4(ipv4) if !ipv4.is_loopback() && !ipv4.is_link_local() => {
                    ips.push(IpAddr::V4(ipv4));
                }
                IpAddr::V6(ipv6) if !ipv6.is_loopback() && !ipv6.is_unspecified() => {
                    ips.push(IpAddr::V6(ipv6));
                }
                _ => continue,
            }
        }

        if !ips.is_empty() {
            result.push((interface.name.clone(), ips));
        }
    }

    result
}

pub fn display_interfaces() {
    let interfaces = list_network_interfaces();
    
    if interfaces.is_empty() {
        println!("No active network interfaces found.");
        return;
    }

    println!("Active Network Interfaces:");
    println!("{:-<40}", "");
    
    for (name, ips) in interfaces {
        println!("Interface: {}", name);
        for ip in ips {
            println!("  - {}", ip);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_listing() {
        let interfaces = list_network_interfaces();
        // At minimum, loopback should be filtered out
        for (name, ips) in interfaces {
            assert!(!name.contains("lo"));
            assert!(!ips.is_empty());
            
            for ip in ips {
                match ip {
                    IpAddr::V4(ipv4) => {
                        assert!(!ipv4.is_loopback());
                        assert!(!ipv4.is_link_local());
                    }
                    IpAddr::V6(ipv6) => {
                        assert!(!ipv6.is_loopback());
                        assert!(!ipv6.is_unspecified());
                    }
                }
            }
        }
    }
}