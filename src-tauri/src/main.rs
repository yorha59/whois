#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

// ── Service type enum (W-2: expanded) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    SSH,
    HTTP,
    HTTPS,
    FTP,
    Telnet,
    SMTP,
    DNS,
    DHCP,
    Redis,
    PostgreSQL,
    MySQL,
    MongoDB,
    MQTT,
    SMB,
    RDP,
    VNC,
    Docker,
    Kubernetes,
    Elasticsearch,
    Grafana,
    Prometheus,
    MinIO,
    Gitea,
    Unknown,
}

impl ServiceType {
    pub fn from_port(port: u16) -> Self {
        match port {
            21 => ServiceType::FTP,
            22 => ServiceType::SSH,
            23 => ServiceType::Telnet,
            25 | 587 | 465 => ServiceType::SMTP,
            53 => ServiceType::DNS,
            67 | 68 => ServiceType::DHCP,
            80 => ServiceType::HTTP,
            443 => ServiceType::HTTPS,
            445 => ServiceType::SMB,
            1883 | 8883 => ServiceType::MQTT,
            2375 | 2376 => ServiceType::Docker,
            3000 => ServiceType::Gitea, // or Grafana, common for both
            3306 => ServiceType::MySQL,
            3389 => ServiceType::RDP,
            5432 => ServiceType::PostgreSQL,
            5900 | 5901 => ServiceType::VNC,
            6379 => ServiceType::Redis,
            6443 => ServiceType::Kubernetes,
            8080 | 8443 | 8888 | 5000 | 5173 => ServiceType::HTTP,
            9000 => ServiceType::MinIO,
            9090 => ServiceType::Prometheus,
            9200 | 9300 => ServiceType::Elasticsearch,
            27017 => ServiceType::MongoDB,
            _ => ServiceType::Unknown,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ServiceType::SSH => "SSH",
            ServiceType::HTTP => "HTTP",
            ServiceType::HTTPS => "HTTPS",
            ServiceType::FTP => "FTP",
            ServiceType::Telnet => "Telnet",
            ServiceType::SMTP => "SMTP",
            ServiceType::DNS => "DNS",
            ServiceType::DHCP => "DHCP",
            ServiceType::Redis => "Redis",
            ServiceType::PostgreSQL => "PostgreSQL",
            ServiceType::MySQL => "MySQL",
            ServiceType::MongoDB => "MongoDB",
            ServiceType::MQTT => "MQTT",
            ServiceType::SMB => "SMB",
            ServiceType::RDP => "RDP",
            ServiceType::VNC => "VNC",
            ServiceType::Docker => "Docker",
            ServiceType::Kubernetes => "K8s API",
            ServiceType::Elasticsearch => "Elasticsearch",
            ServiceType::Grafana => "Grafana",
            ServiceType::Prometheus => "Prometheus",
            ServiceType::MinIO => "MinIO",
            ServiceType::Gitea => "Gitea",
            ServiceType::Unknown => "Unknown",
        }
    }
}

// ── Data structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub service: ServiceType,
    pub service_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub ip: String,
    pub hostname: Option<String>,
    pub ports: Vec<PortInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub local_ip: String,
    pub subnet: String,
}

// ── W-2: Expanded port list (Top 30+) ──────────────────────────────────────

const COMMON_PORTS: &[u16] = &[
    21,    // FTP
    22,    // SSH
    23,    // Telnet
    25,    // SMTP
    53,    // DNS
    80,    // HTTP
    443,   // HTTPS
    445,   // SMB
    587,   // SMTP (submission)
    1883,  // MQTT
    2375,  // Docker
    3000,  // Gitea / Grafana
    3306,  // MySQL
    3389,  // RDP
    5000,  // HTTP (Flask/registry)
    5432,  // PostgreSQL
    5900,  // VNC
    6379,  // Redis
    6443,  // Kubernetes API
    8080,  // HTTP alt
    8443,  // HTTPS alt
    8883,  // MQTT TLS
    8888,  // HTTP alt
    9000,  // MinIO
    9090,  // Prometheus
    9200,  // Elasticsearch
    9300,  // Elasticsearch transport
    27017, // MongoDB
];

// ── Port scanning ──────────────────────────────────────────────────────────

async fn scan_port(ip: IpAddr, port: u16) -> Option<PortInfo> {
    let addr = SocketAddr::new(ip, port);
    let timeout_duration = Duration::from_millis(500);

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => {
            let svc = ServiceType::from_port(port);
            let label = svc.label().to_string();
            Some(PortInfo {
                port,
                service: svc,
                service_label: label,
            })
        }
        _ => None,
    }
}

// ── W-3: Hostname resolution ───────────────────────────────────────────────

fn resolve_hostname(ip: &str) -> Option<String> {
    use std::net::ToSocketAddrs;
    // Try reverse DNS lookup
    let socket_str = format!("{}:0", ip);
    if let Ok(mut addrs) = socket_str.to_socket_addrs() {
        if let Some(_addr) = addrs.next() {
            // Try to get the hostname via DNS
            if let Ok(host) = dns_lookup_reverse(ip) {
                if host != ip {
                    return Some(host);
                }
            }
        }
    }
    None
}

fn dns_lookup_reverse(ip: &str) -> Result<String, ()> {
    use std::process::Command;
    // Use system's `host` command for reverse DNS
    let output = Command::new("host")
        .arg(ip)
        .output()
        .map_err(|_| ())?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "X.X.X.X.in-addr.arpa domain name pointer hostname."
        if let Some(line) = stdout.lines().next() {
            if line.contains("domain name pointer") {
                if let Some(hostname) = line.split("domain name pointer ").nth(1) {
                    let hostname = hostname.trim_end_matches('.');
                    return Ok(hostname.to_string());
                }
            }
        }
    }
    Err(())
}

// ── W-1: Auto-detect local subnet via Rust ─────────────────────────────────

#[tauri::command]
fn detect_network() -> Result<NetworkInfo, String> {
    // Create a UDP socket and "connect" to an external address
    // This doesn't actually send data but lets us find our local IP
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind socket: {}", e))?;
    
    socket.connect("8.8.8.8:53")
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let local_addr = socket.local_addr()
        .map_err(|e| format!("Failed to get local addr: {}", e))?;
    
    let local_ip = local_addr.ip().to_string();
    
    // Extract subnet (first 3 octets for /24)
    let parts: Vec<&str> = local_ip.split('.').collect();
    if parts.len() == 4 {
        let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        Ok(NetworkInfo {
            local_ip,
            subnet,
        })
    } else {
        Err("Invalid IP format".to_string())
    }
}

// ── Main scan function (improved) ──────────────────────────────────────────

#[tauri::command]
async fn perform_real_scan(subnet: String, extra_ports: Option<Vec<u16>>) -> Vec<HostInfo> {
    let mut results = Vec::new();
    
    // Merge common ports with any user-specified extra ports (W-5)
    let mut ports_to_scan: Vec<u16> = COMMON_PORTS.to_vec();
    if let Some(extra) = extra_ports {
        for p in extra {
            if !ports_to_scan.contains(&p) {
                ports_to_scan.push(p);
            }
        }
    }
    ports_to_scan.sort();
    
    let mut handles = Vec::new();

    for i in 1..255 {
        let ip_str = format!("{}.{}", subnet, i);
        let ip: IpAddr = match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => continue,
        };
        let ports = ports_to_scan.clone();

        handles.push(tokio::spawn(async move {
            let mut found_ports = Vec::new();
            
            // Scan all ports concurrently per host
            let mut port_handles = Vec::new();
            for port in ports {
                let ip_clone = ip;
                port_handles.push(tokio::spawn(async move {
                    scan_port(ip_clone, port).await
                }));
            }
            
            for ph in port_handles {
                if let Ok(Some(info)) = ph.await {
                    found_ports.push(info);
                }
            }
            
            if !found_ports.is_empty() {
                // Sort ports numerically
                found_ports.sort_by_key(|p| p.port);
                
                // W-3: Try hostname resolution
                let hostname = resolve_hostname(&ip_str);
                
                Some(HostInfo {
                    ip: ip_str,
                    hostname,
                    ports: found_ports,
                })
            } else {
                None
            }
        }));
    }

    for handle in handles {
        if let Ok(Some(host)) = handle.await {
            results.push(host);
        }
    }

    // Sort by IP address
    results.sort_by(|a, b| {
        let a_parts: Vec<u8> = a.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u8> = b.ip.split('.').filter_map(|s| s.parse().ok()).collect();
        a_parts.cmp(&b_parts)
    });

    results
}

// ── App entry point ────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_network,
            perform_real_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
