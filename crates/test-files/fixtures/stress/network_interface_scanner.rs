use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use pnet_datalink;

pub fn list_network_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    
    for iface in pnet_datalink::interfaces() {
        let mut info = format!("Interface: {}", iface.name);
        
        if let Some(mac) = iface.mac {
            info.push_str(&format!("\n  MAC: {}", mac));
        }
        
        for ip in iface.ips {
            match ip.ip() {
                IpAddr::V4(ipv4) => {
                    info.push_str(&format!("\n  IPv4: {}", ipv4));
                }
                IpAddr::V6(ipv6) => {
                    info.push_str(&format!("\n  IPv6: {}", ipv6));
                }
            }
        }
        
        if iface.is_up() {
            info.push_str("\n  Status: UP");
        } else {
            info.push_str("\n  Status: DOWN");
        }
        
        if iface.is_loopback() {
            info.push_str("\n  Type: Loopback");
        } else if iface.is_broadcast() {
            info.push_str("\n  Type: Broadcast");
        } else if iface.is_point_to_point() {
            info.push_str("\n  Type: Point-to-Point");
        }
        
        if let Some(mtu) = iface.mtu {
            info.push_str(&format!("\n  MTU: {}", mtu));
        }
        
        interfaces.push(info);
    }
    
    interfaces
}

pub fn find_interface_by_name(name: &str) -> Option<String> {
    for iface in pnet_datalink::interfaces() {
        if iface.name == name {
            let mut info = format!("Found interface: {}", name);
            
            if let Some(mac) = iface.mac {
                info.push_str(&format!("\n  MAC: {}", mac));
            }
            
            for ip in iface.ips {
                match ip.ip() {
                    IpAddr::V4(ipv4) => info.push_str(&format!("\n  IPv4: {}", ipv4)),
                    IpAddr::V6(ipv6) => info.push_str(&format!("\n  IPv6: {}", ipv6)),
                }
            }
            
            return Some(info);
        }
    }
    None
}

pub fn get_active_interfaces() -> Vec<String> {
    pnet_datalink::interfaces()
        .iter()
        .filter(|iface| iface.is_up() && !iface.is_loopback())
        .map(|iface| iface.name.clone())
        .collect()
}

pub fn has_ipv4_address(interface_name: &str) -> bool {
    pnet_datalink::interfaces()
        .iter()
        .find(|iface| iface.name == interface_name)
        .map_or(false, |iface| {
            iface.ips.iter().any(|ip| matches!(ip.ip(), IpAddr::V4(_)))
        })
}

pub fn has_ipv6_address(interface_name: &str) -> bool {
    pnet_datalink::interfaces()
        .iter()
        .find(|iface| iface.name == interface_name)
        .map_or(false, |iface| {
            iface.ips.iter().any(|ip| matches!(ip.ip(), IpAddr::V6(_)))
        })
}