use serde::{Deserialize, Serialize};
use std::fs;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::process::Command;
use std::sync::Arc;
use surge_ping::{Client, Config};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

// ── ARP 扫描：发现 IoT 设备（链路层发现，无法被防火墙阻止） ─────────────────

/// ARP 扫描结果：IP -> MAC 地址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpEntry {
    pub ip: String,
    pub mac: String,
    pub interface: Option<String>,
}

/// 执行 ARP 扫描发现本地网络中的设备
/// Linux 优先直接读取 `/proc/net/arp`，macOS 优先使用系统自带 `arp`。
pub fn arp_scan(subnet: &str) -> Vec<ArpEntry> {
    #[cfg(target_os = "linux")]
    {
        return scan_linux_arp_cache(subnet);
    }

    #[cfg(target_os = "macos")]
    {
        return scan_macos_arp_cache(subnet);
    }

    #[allow(unreachable_code)]
    Vec::new()
}

#[cfg(target_os = "linux")]
fn scan_linux_arp_cache(subnet: &str) -> Vec<ArpEntry> {
    read_arp_cache_file("/proc/net/arp", subnet, parse_proc_net_arp).unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn scan_macos_arp_cache(subnet: &str) -> Vec<ArpEntry> {
    let command_entries = ["/usr/sbin/arp", "/sbin/arp"]
        .into_iter()
        .find_map(|path| run_arp_command(path, &["-an"], subnet));

    command_entries.unwrap_or_else(|| {
        ["/private/var/run/arp", "/var/run/arp"]
            .into_iter()
            .find_map(|path| read_arp_cache_file(path, subnet, parse_arp_output))
            .unwrap_or_default()
    })
}

fn run_arp_command(path: &str, args: &[&str], subnet: &str) -> Option<Vec<ArpEntry>> {
    let output = Command::new(path).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(parse_arp_output(&stdout, subnet))
}

fn read_arp_cache_file<F>(path: &str, subnet: &str, parser: F) -> Option<Vec<ArpEntry>>
where
    F: Fn(&str, &str) -> Vec<ArpEntry>,
{
    let contents = fs::read_to_string(path).ok()?;
    Some(parser(&contents, subnet))
}

/// 解析 macOS 风格的 arp -a 输出
/// 格式: ? (192.168.1.1) at ab:cd:ef:12:34:56 on en0 [ethernet]
pub fn parse_arp_output(output: &str, subnet: &str) -> Vec<ArpEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        // macOS 格式: ? (192.168.1.1) at ab:cd:ef:12:34:56 on en0
        // Linux 格式: ? (192.168.1.1) at ab:cd:ef:12:34:56 [ether] on eth0
        if let Some(ip_start) = line.find('(') {
            if let Some(ip_end) = line.find(')') {
                let ip = line[ip_start + 1..ip_end].to_string();

                // 只保留指定子网的 IP
                if !ip.starts_with(subnet) {
                    continue;
                }

                // 提取 MAC 地址
                if let Some(at_pos) = line.find("at ") {
                    let after_at = &line[at_pos + 3..];
                    let mac = after_at.split_whitespace().next().unwrap_or("").to_string();

                    // 验证 MAC 地址格式
                    if mac.len() >= 17 && mac.contains(':') {
                        // 提取接口名
                        let interface = line
                            .split(" on ")
                            .nth(1)
                            .map(|s| s.split_whitespace().next().unwrap_or("").to_string());

                        entries.push(ArpEntry { ip, mac, interface });
                    }
                }
            }
        }
    }

    entries
}

/// 解析 Linux ip neigh 输出
/// 格式: 192.168.1.1 dev eth0 lladdr ab:cd:ef:12:34:56 REACHABLE
pub fn parse_ip_neigh_output(output: &str, subnet: &str) -> Vec<ArpEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let ip = parts[0].to_string();

            // 只保留指定子网的 IP
            if !ip.starts_with(subnet) {
                continue;
            }

            // 查找 lladdr 后的 MAC 地址
            if let Some(lladdr_pos) = parts.iter().position(|&p| p == "lladdr") {
                if let Some(mac) = parts.get(lladdr_pos + 1) {
                    // 获取接口名 (dev 后的值)
                    let interface = parts.get(2).map(|s| s.to_string());

                    entries.push(ArpEntry {
                        ip,
                        mac: mac.to_string(),
                        interface,
                    });
                }
            }
        }
    }

    entries
}

/// 解析 Linux `/proc/net/arp`
/// 格式:
/// IP address       HW type     Flags       HW address            Mask     Device
/// 192.168.1.1      0x1         0x2         ab:cd:ef:12:34:56     *        eth0
pub fn parse_proc_net_arp(output: &str, subnet: &str) -> Vec<ArpEntry> {
    let mut entries = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let ip = parts[0];
        if !ip.starts_with(subnet) {
            continue;
        }

        let flags = parts[2];
        let mac = parts[3];
        if flags == "0x0" || mac == "00:00:00:00:00:00" || mac == "(incomplete)" {
            continue;
        }

        entries.push(ArpEntry {
            ip: ip.to_string(),
            mac: mac.to_string(),
            interface: Some(parts[5].to_string()),
        });
    }

    entries
}

/// 发送 ARP 请求主动发现设备（使用 arping）
/// 使用并发控制以提高速度，同时避免系统资源耗尽
pub async fn active_arp_scan(subnet: &str) -> Vec<ArpEntry> {
    let mut entries = Vec::new();
    let semaphore = Arc::new(Semaphore::new(50)); // 限制并发 ARP 请求数量
    let mut handles = Vec::new();

    // 对子网内的每个 IP 发送 ARP 请求
    for i in 1..255 {
        let ip = format!("{}.{}", subnet, i);
        let sem_clone = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.ok()?;
            // 使用 spawn_blocking 因为 arping 是阻塞的系统调用
            let result = tokio::task::spawn_blocking(move || send_arp_request(&ip))
                .await
                .ok()?;
            result
        }));
    }

    for handle in handles {
        if let Ok(Some(entry)) = handle.await {
            entries.push(entry);
        }
    }

    entries
}

/// 发送单个 ARP 请求
fn send_arp_request(ip: &str) -> Option<ArpEntry> {
    // 尝试使用 arping（需要 root 权限）
    let output = if cfg!(target_os = "macos") {
        // macOS: arping -c 1 -t 500 <ip>
        Command::new("arping")
            .args(&["-c", "1", "-t", "500", ip])
            .output()
    } else {
        // Linux: arping -c 1 -w 500000 <ip>
        Command::new("arping")
            .args(&["-c", "1", "-w", "500000", ip])
            .output()
    };

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 解析 arping 输出获取 MAC 地址
            if let Some(mac) = parse_arping_output(&stdout) {
                return Some(ArpEntry {
                    ip: ip.to_string(),
                    mac,
                    interface: None,
                });
            }
        }
    }

    None
}

/// 解析 arping 输出提取 MAC 地址
pub fn parse_arping_output(output: &str) -> Option<String> {
    // 典型输出: 64 bytes from 192.168.1.1 (ab:cd:ef:12:34:56): icmp_seq=0
    for line in output.lines() {
        if let Some(open) = line.find('(') {
            if let Some(close) = line.find(')') {
                let mac = &line[open + 1..close];
                if mac.contains(':') && mac.len() == 17 {
                    return Some(mac.to_string());
                }
            }
        }
    }
    None
}

// ── ICMP Ping 主机存活检测 ──────────────────────────────────────────────────

/// Ping 一个主机，返回是否在线（支持重试机制）
/// 默认超时 500ms，重试 2 次（总共 3 次尝试）
pub async fn ping_host(ip: &str) -> bool {
    ping_host_with_retries(ip, 500, 2).await
}

/// Ping 一个主机，支持自定义重试次数
/// timeout_ms: 每次超时时间
/// retries: 重试次数（0 表示只尝试 1 次）
pub async fn ping_host_with_retries(ip: &str, timeout_ms: u64, retries: u32) -> bool {
    let ip_addr: IpAddr = match ip.parse() {
        Ok(addr) => addr,
        Err(_) => return false,
    };

    // 暂不支持 IPv6
    if matches!(ip_addr, IpAddr::V6(_)) {
        return false;
    }

    let config = Config::default();
    let client = match Client::new(&config) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // surge-ping API: pinger 是异步方法，返回 Pinger 结构体（非 Result）
    let ident = surge_ping::PingIdentifier(rand::random());
    let mut pinger = client.pinger(ip_addr, ident).await;
    pinger.timeout(Duration::from_millis(timeout_ms));

    // 尝试多次 ping
    for i in 0..=retries {
        let seq = surge_ping::PingSequence(i as u16);
        match pinger.ping(seq, &[]).await {
            Ok((_, _)) => return true,
            Err(_) => {
                // 最后一次失败才返回 false
                if i == retries {
                    return false;
                }
                // 短暂延迟后重试
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }

    false
}

// ── UDP Service Discovery (mDNS + SSDP) ────────────────────────────────────

/// mDNS 服务发现结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsServiceInfo {
    pub service_type: String,
    pub name: String,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub ip: Option<String>,
    pub txt_records: Vec<String>,
}

/// SSDP/UPnP 设备发现结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsdpDeviceInfo {
    pub device_type: String,
    pub location: Option<String>,
    pub server: Option<String>,
    pub usn: Option<String>,
    pub ip: String,
}

/// 服务发现结果汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryResult {
    pub ip: String,
    pub mdns_services: Vec<MdnsServiceInfo>,
    pub ssdp_devices: Vec<SsdpDeviceInfo>,
}

impl ServiceDiscoveryResult {
    pub fn new(ip: String) -> Self {
        Self {
            ip,
            mdns_services: Vec::new(),
            ssdp_devices: Vec::new(),
        }
    }
}

/// mDNS 查询目标
const MDNS_QUERY_TARGETS: &[(&str, &str)] = &[
    ("_hap._tcp.local", "HomeKit"),
    ("_miio._udp.local", "Xiaomi MiIO"),
    ("_yeelight._tcp.local", "Yeelight"),
    ("_googlecast._tcp.local", "Google Cast"),
    ("_airplay._tcp.local", "AirPlay"),
    ("_raop._tcp.local", "AirTunes RAOP"),
    ("_sonos._tcp.local", "Sonos"),
    ("_http._tcp.local", "HTTP"),
    ("_printer._tcp.local", "Printer"),
    ("_ipp._tcp.local", "IPP Printer"),
];

/// 构建 mDNS 查询包 (DNS-SD)
pub fn build_mdns_query(service: &str) -> Vec<u8> {
    let mut packet = Vec::new();

    // Transaction ID
    packet.extend_from_slice(&[0x00, 0x00]);
    // Flags: Standard query
    packet.extend_from_slice(&[0x00, 0x00]);
    // Questions: 1
    packet.extend_from_slice(&[0x00, 0x01]);
    // Answer RRs: 0
    packet.extend_from_slice(&[0x00, 0x00]);
    // Authority RRs: 0
    packet.extend_from_slice(&[0x00, 0x00]);
    // Additional RRs: 0
    packet.extend_from_slice(&[0x00, 0x00]);

    // Query name
    for part in service.split('.') {
        packet.push(part.len() as u8);
        packet.extend_from_slice(part.as_bytes());
    }
    packet.push(0x00);

    // Query Type: PTR (12)
    packet.extend_from_slice(&[0x00, 0x0C]);
    // Query Class: IN (1), with Unicast Response bit
    packet.extend_from_slice(&[0x00, 0x01]);

    packet
}

/// 解析 mDNS 响应包
pub fn parse_mdns_response(data: &[u8], src_ip: &str) -> Vec<MdnsServiceInfo> {
    let mut services = Vec::new();

    if data.len() < 12 {
        return services;
    }

    // Parse header
    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let ancount = u16::from_be_bytes([data[6], data[7]]);

    if ancount == 0 {
        return services;
    }

    // Skip header
    let mut offset = 12;

    // Skip questions
    for _ in 0..qdcount {
        while offset < data.len() && data[offset] != 0 {
            let len = data[offset] as usize;
            if len & 0xC0 == 0xC0 {
                // Compressed pointer
                offset += 2;
                break;
            }
            offset += len + 1;
        }
        if offset < data.len() && data[offset] == 0 {
            offset += 1;
        }
        // Skip QTYPE and QCLASS
        offset += 4;
    }

    // Parse answers (simplified)
    // In a real implementation, we'd fully parse all DNS records
    // For now, extract service info from SRV and TXT records if present

    // Look for service patterns in the raw data
    let data_str = String::from_utf8_lossy(data);

    // Try to extract service names
    for (service_type, label) in MDNS_QUERY_TARGETS {
        if data_str.contains(service_type.trim_end_matches(".local")) {
            let mut info = MdnsServiceInfo {
                service_type: label.to_string(),
                name: String::new(),
                hostname: None,
                port: None,
                ip: Some(src_ip.to_string()),
                txt_records: Vec::new(),
            };

            // Try to extract instance name
            if let Some(idx) = data_str.find(service_type) {
                // Look backwards for the instance name
                let before = &data_str[..idx];
                if let Some(last_dot) = before.rfind('.') {
                    if let Some(second_last_dot) = before[..last_dot].rfind('.') {
                        info.name = before[second_last_dot + 1..last_dot].to_string();
                    }
                }
            }

            services.push(info);
        }
    }

    services
}

/// 执行 mDNS 服务发现
pub async fn mdns_discovery(timeout_ms: u64) -> Vec<MdnsServiceInfo> {
    let mut all_services = Vec::new();

    // Try to bind to multicast port 5353, or use any available port
    let socket = match UdpSocket::bind("0.0.0.0:5353") {
        Ok(s) => s,
        Err(_) => match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(_) => return all_services,
        },
    };

    // Enable broadcast
    let _ = socket.set_broadcast(true);

    // Set timeouts
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(timeout_ms)));
    let _ = socket.set_write_timeout(Some(std::time::Duration::from_millis(1000)));

    // Join multicast group for mDNS
    // Note: This may require elevated permissions on some systems

    // Send queries for each service type
    let mdns_addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(224, 0, 0, 251), 5353);

    for (service, _) in MDNS_QUERY_TARGETS {
        let query = build_mdns_query(service);
        let _ = socket.send_to(&query, mdns_addr);
    }

    // Also send a general browse query for all services
    let browse_query = build_mdns_query("_services._dns-sd._udp.local");
    let _ = socket.send_to(&browse_query, mdns_addr);

    // Collect responses
    let start = std::time::Instant::now();
    let mut buf = [0u8; 4096];

    while start.elapsed().as_millis() < timeout_ms as u128 {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                let services = parse_mdns_response(&buf[..len], &src.ip().to_string());
                all_services.extend(services);
            }
            Err(_) => {
                // Timeout or error, continue
                break;
            }
        }
    }

    // Remove duplicates based on service type and IP
    all_services.sort_by(|a, b| {
        (a.service_type.clone(), a.ip.clone()).cmp(&(b.service_type.clone(), b.ip.clone()))
    });
    all_services.dedup_by(|a, b| a.service_type == b.service_type && a.ip == b.ip);

    all_services
}

/// 构建 SSDP M-SEARCH 请求
pub fn build_ssdp_search() -> String {
    format!(
        "M-SEARCH * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         MAN: \"ssdp:discover\"\r\n\
         MX: 3\r\n\
         ST: ssdp:all\r\n\
         \r\n"
    )
}

/// 解析 SSDP 响应
pub fn parse_ssdp_response(data: &[u8], src_ip: &str) -> Option<SsdpDeviceInfo> {
    let response = String::from_utf8_lossy(data);
    let lines: Vec<&str> = response.lines().collect();

    if lines.is_empty() || !lines[0].contains("200 OK") {
        return None;
    }

    let mut device_type = String::from("Unknown");
    let mut location = None;
    let mut server = None;
    let mut usn = None;

    for line in &lines[1..] {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();

            match key.as_str() {
                "st" | "nt" => device_type = value,
                "location" => location = Some(value),
                "server" => server = Some(value),
                "usn" => usn = Some(value),
                _ => {}
            }
        }
    }

    Some(SsdpDeviceInfo {
        device_type,
        location,
        server,
        usn,
        ip: src_ip.to_string(),
    })
}

/// 执行 SSDP/UPnP 设备发现
pub async fn ssdp_discovery(timeout_ms: u64) -> Vec<SsdpDeviceInfo> {
    let mut devices = Vec::new();

    // Bind to any available port
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return devices,
    };

    // Enable broadcast
    let _ = socket.set_broadcast(true);

    // Set timeouts
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(timeout_ms)));
    let _ = socket.set_write_timeout(Some(std::time::Duration::from_millis(1000)));

    // Send M-SEARCH to multicast address
    let ssdp_addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(239, 255, 255, 250), 1900);
    let request = build_ssdp_search();

    // Send multiple times to increase discovery rate
    for _ in 0..3 {
        let _ = socket.send_to(request.as_bytes(), ssdp_addr);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Collect responses
    let start = std::time::Instant::now();
    let mut buf = [0u8; 4096];

    while start.elapsed().as_millis() < timeout_ms as u128 {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(device) = parse_ssdp_response(&buf[..len], &src.ip().to_string()) {
                    devices.push(device);
                }
            }
            Err(_) => {
                // Timeout or error, continue
                break;
            }
        }
    }

    // Remove duplicates based on USN
    devices.sort_by(|a, b| {
        a.usn
            .clone()
            .unwrap_or_default()
            .cmp(&b.usn.clone().unwrap_or_default())
    });
    devices.dedup_by(|a, b| a.usn == b.usn && a.ip == b.ip);

    devices
}

/// 对特定 IP 执行服务发现
pub async fn discover_services_for_host(ip: &str, timeout_ms: u64) -> ServiceDiscoveryResult {
    let mut result = ServiceDiscoveryResult::new(ip.to_string());

    // For now, we use the global discovery and filter by IP
    // In a future enhancement, we could target specific IPs

    let mdns_services = mdns_discovery(timeout_ms / 2).await;
    for service in mdns_services {
        if service.ip.as_deref() == Some(ip) {
            result.mdns_services.push(service);
        }
    }

    let ssdp_devices = ssdp_discovery(timeout_ms / 2).await;
    for device in ssdp_devices {
        if device.ip == ip {
            result.ssdp_devices.push(device);
        }
    }

    result
}

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
    SSDP,
    MDNS,
    LLMNR,
    Yeelight,
    XiaomiGateway,
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
            4321 | 9898 => ServiceType::Yeelight,
            5353 => ServiceType::MDNS,
            5357 => ServiceType::LLMNR,
            5432 => ServiceType::PostgreSQL,
            5900 | 5901 => ServiceType::VNC,
            6379 => ServiceType::Redis,
            6443 => ServiceType::Kubernetes,
            8080 | 8443 | 8888 | 5000 | 5173 => ServiceType::HTTP,
            8083 | 8084 | 8245 => ServiceType::XiaomiGateway,
            9000 => ServiceType::MinIO,
            9090 => ServiceType::Prometheus,
            9200 | 9300 => ServiceType::Elasticsearch,
            1900 => ServiceType::SSDP,
            27017 => ServiceType::MongoDB,
            54321 => ServiceType::XiaomiGateway,
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
            ServiceType::SSDP => "SSDP/UPnP",
            ServiceType::MDNS => "mDNS",
            ServiceType::LLMNR => "LLMNR",
            ServiceType::Yeelight => "Yeelight",
            ServiceType::XiaomiGateway => "Xiaomi Gateway",
            ServiceType::Unknown => "Unknown",
        }
    }
}

// ── Data structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub timeout_ms: u64,
    pub max_concurrent: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 1000,
            max_concurrent: 100,
        }
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns_services: Option<Vec<MdnsServiceInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssdp_devices: Option<Vec<SsdpDeviceInfo>>,
}

impl HostInfo {
    pub fn new(ip: String) -> Self {
        Self {
            ip,
            hostname: None,
            ports: Vec::new(),
            mdns_services: None,
            ssdp_devices: None,
        }
    }
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
    1900,  // SSDP/UPnP
    2375,  // Docker
    3000,  // Gitea / Grafana
    3306,  // MySQL
    3389,  // RDP
    4321,  // Yeelight
    5000,  // HTTP (Flask/registry)
    5353,  // mDNS
    5357,  // LLMNR
    5432,  // PostgreSQL
    5900,  // VNC
    6379,  // Redis
    6443,  // Kubernetes API
    8080,  // HTTP alt
    8083,  // Xiaomi
    8084,  // Xiaomi
    8245,  // Xiaomi
    8443,  // HTTPS alt
    8883,  // MQTT TLS
    8888,  // HTTP alt
    9000,  // MinIO
    9090,  // Prometheus
    9200,  // Elasticsearch
    9300,  // Elasticsearch transport
    9898,  // Yeelight
    27017, // MongoDB
    54321, // Xiaomi Gateway
];

// ── Port scanning ──────────────────────────────────────────────────────────

async fn scan_port(ip: IpAddr, port: u16, timeout_ms: u64) -> Option<PortInfo> {
    let addr = SocketAddr::new(ip, port);
    let timeout_duration = Duration::from_millis(timeout_ms);

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

pub fn resolve_hostname(ip: &str) -> Option<String> {
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

pub fn dns_lookup_reverse(ip: &str) -> Result<String, ()> {
    use std::process::Command;
    // Use system's `host` command for reverse DNS
    let output = Command::new("host").arg(ip).output().map_err(|_| ())?;

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

pub fn detect_network_internal() -> Result<NetworkInfo, String> {
    // Create a UDP socket and "connect" to an external address
    // This doesn't actually send data but lets us find our local IP
    let socket =
        UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind socket: {}", e))?;

    socket
        .connect("8.8.8.8:53")
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let local_addr = socket
        .local_addr()
        .map_err(|e| format!("Failed to get local addr: {}", e))?;

    let local_ip = local_addr.ip().to_string();

    // Extract subnet (first 3 octets for /24)
    let parts: Vec<&str> = local_ip.split('.').collect();
    if parts.len() == 4 {
        let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        Ok(NetworkInfo { local_ip, subnet })
    } else {
        Err("Invalid IP format".to_string())
    }
}

// ── Main scan function (improved with ARP scan + ICMP ping + port scan) ────

pub async fn perform_real_scan_internal(
    subnet: String,
    extra_ports: Option<Vec<u16>>,
    config: Option<ScanConfig>,
) -> Vec<HostInfo> {
    let mut results = Vec::new();
    let config = config.unwrap_or_default();

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

    // ═══════════════════════════════════════════════════════════════════════════
    // Phase 1: ARP 扫描（发现链路层设备，IoT 设备无法阻止 ARP）
    // ═══════════════════════════════════════════════════════════════════════════

    println!("Phase 1: ARP scan to discover all link-layer devices...");

    // 1a. 被动 ARP 缓存扫描
    let arp_entries = arp_scan(&subnet);
    println!("  Passive ARP cache: {} devices found", arp_entries.len());

    // 1b. 主动 ARP 探测（使用 arping）- 这对静态 IP 设备特别重要
    // 因为静态 IP 设备可能没有出现在 ARP 缓存中
    println!("  Active ARP probing (arping) for static IP devices...");
    let active_entries = active_arp_scan(&subnet).await;
    println!(
        "  Active ARP probe: {} devices responded",
        active_entries.len()
    );

    // 合并被动和主动 ARP 结果
    let mut all_arp_entries = arp_entries.clone();
    for entry in active_entries {
        // 避免重复添加
        if !all_arp_entries.iter().any(|e| e.ip == entry.ip) {
            all_arp_entries.push(entry);
        }
    }
    println!(
        "Phase 1 complete: {} total devices found via ARP",
        all_arp_entries.len()
    );

    // 收集 ARP 发现的 IP 地址
    let mut online_hosts: Vec<String> = all_arp_entries.iter().map(|e| e.ip.clone()).collect();

    // ═══════════════════════════════════════════════════════════════════════════
    // Phase 2: ICMP Ping 补充检测（捕获那些 ARP 缓存中没有但有响应的设备）
    // ═══════════════════════════════════════════════════════════════════════════

    println!("Phase 2: ICMP Ping scan to detect additional online hosts...");
    println!("  (Using 3 retries per host with 500ms timeout for reliability)");
    let mut ping_handles = Vec::new();

    for i in 1..255 {
        let ip_str = format!("{}.{}", subnet, i);

        // 跳过已经通过 ARP 发现的设备
        if online_hosts.contains(&ip_str) {
            continue;
        }

        ping_handles.push(tokio::spawn(async move {
            // 使用重试机制：500ms 超时，重试 2 次（总共 3 次尝试）
            if ping_host_with_retries(&ip_str, 500, 2).await {
                println!("    ICMP response from: {}", ip_str);
                Some(ip_str)
            } else {
                None
            }
        }));
    }

    let mut icmp_hosts = Vec::new();
    for handle in ping_handles {
        if let Ok(Some(ip)) = handle.await {
            online_hosts.push(ip.clone());
            icmp_hosts.push(ip);
        }
    }

    println!(
        "Phase 2 complete: {} new hosts via ICMP ping ({} total online)",
        icmp_hosts.len(),
        online_hosts.len()
    );

    // ═══════════════════════════════════════════════════════════════════════════
    // Phase 3: UDP Service Discovery (mDNS + SSDP)
    // ═══════════════════════════════════════════════════════════════════════════

    println!("Phase 3: UDP Service Discovery (mDNS + SSDP)...");

    // Run mDNS discovery
    let mdns_results = mdns_discovery(2000).await;
    println!("  mDNS: Found {} service(s)", mdns_results.len());
    for service in &mdns_results {
        println!(
            "    - {} ({}) at {:?}",
            service.service_type, service.name, service.ip
        );
    }

    // Run SSDP discovery
    let ssdp_results = ssdp_discovery(2000).await;
    println!("  SSDP: Found {} device(s)", ssdp_results.len());
    for device in &ssdp_results {
        println!("    - {} at {}", device.device_type, device.ip);
    }

    // Create a map of discovered services by IP
    let mut services_by_ip: std::collections::HashMap<String, ServiceDiscoveryResult> =
        std::collections::HashMap::new();

    for service in mdns_results {
        if let Some(ip) = &service.ip {
            let entry = services_by_ip
                .entry(ip.clone())
                .or_insert_with(|| ServiceDiscoveryResult::new(ip.clone()));
            entry.mdns_services.push(service);
        }
    }

    for device in ssdp_results {
        let entry = services_by_ip
            .entry(device.ip.clone())
            .or_insert_with(|| ServiceDiscoveryResult::new(device.ip.clone()));
        entry.ssdp_devices.push(device);
    }

    println!(
        "Phase 3 complete: {} host(s) with discovered services",
        services_by_ip.len()
    );

    // ═══════════════════════════════════════════════════════════════════════════
    // Phase 4: 对发现的设备进行端口扫描
    // ═══════════════════════════════════════════════════════════════════════════

    println!(
        "Phase 4: Port scanning {} hosts with {} common ports...",
        online_hosts.len(),
        ports_to_scan.len()
    );
    println!("  Port list: {:?}", ports_to_scan);

    // Create semaphore to limit concurrent connections
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
    let mut handles = Vec::new();

    // Convert services_by_ip to Arc for sharing across tasks
    let services_by_ip = Arc::new(services_by_ip);

    // 用于跟踪进度的计数器
    let scanned_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let total_hosts = online_hosts.len();

    for ip_str in online_hosts {
        let ip: IpAddr = match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => continue,
        };
        let ports = ports_to_scan.clone();
        let sem_clone = Arc::clone(&semaphore);
        let services_clone = Arc::clone(&services_by_ip);
        let timeout_ms = config.timeout_ms;
        let count_clone = Arc::clone(&scanned_count);

        handles.push(tokio::spawn(async move {
            let mut found_ports = Vec::new();

            // ═══════════════════════════════════════════════════════════════════════════
            // 修复 TrueNAS 端口扫描失败问题：降低并发 + 添加延迟
            // 问题：并发连接触发防火墙临时封锁
            // 解决方案：每批次扫描 5 个端口，批次间延迟 100ms
            // ═══════════════════════════════════════════════════════════════════════════

            const BATCH_SIZE: usize = 5;
            const BATCH_DELAY_MS: u64 = 100;

            for (batch_idx, batch) in ports.chunks(BATCH_SIZE).enumerate() {
                if batch_idx > 0 {
                    // 批次间延迟，避免触发防火墙
                    tokio::time::sleep(Duration::from_millis(BATCH_DELAY_MS)).await;
                }

                // 扫描当前批次的端口
                let mut port_handles = Vec::new();
                for port in batch {
                    let ip_clone = ip;
                    let port = *port;
                    let permit = sem_clone.clone().acquire_owned().await.ok();
                    port_handles.push(tokio::spawn(async move {
                        let _permit = permit; // Hold permit until task completes
                        let result = scan_port(ip_clone, port, timeout_ms).await;
                        // 调试输出：显示每个端口的扫描结果
                        match &result {
                            Some(info) => println!(
                                "      [DEBUG] {}:{} - OPEN ({})",
                                ip_clone, port, info.service_label
                            ),
                            None => {
                                println!("      [DEBUG] {}:{} - closed/filtered", ip_clone, port)
                            }
                        }
                        result
                    }));
                }

                for ph in port_handles {
                    if let Ok(Some(info)) = ph.await {
                        found_ports.push(info);
                    }
                }
            }

            // Sort ports numerically
            found_ports.sort_by_key(|p| p.port);

            // W-3: Try hostname resolution
            let hostname = resolve_hostname(&ip_str);

            // Get discovered services for this host
            let (mdns_services, ssdp_devices) =
                if let Some(discovered) = services_clone.get(&ip_str) {
                    (
                        Some(discovered.mdns_services.clone()),
                        Some(discovered.ssdp_devices.clone()),
                    )
                } else {
                    (None, None)
                };

            // 更新进度并报告发现的端口
            let current = count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if !found_ports.is_empty() {
                let port_list: Vec<String> = found_ports
                    .iter()
                    .map(|p| format!("{}:{}", p.port, p.service_label))
                    .collect();
                println!(
                    "    [{}/{}] {} - found ports: {}",
                    current,
                    total_hosts,
                    ip_str,
                    port_list.join(", ")
                );
            } else {
                println!(
                    "    [{}/{}] {} - no open ports found",
                    current, total_hosts, ip_str
                );
            }

            Some(HostInfo {
                ip: ip_str,
                hostname,
                ports: found_ports,
                mdns_services,
                ssdp_devices,
            })
        }));
    }

    for handle in handles {
        if let Ok(Some(host)) = handle.await {
            // 即使没有发现开放端口，也要包含在线主机（这对静态 IP 设备很重要）
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

// ── W-6: Export scan results ───────────────────────────────────────────────

pub fn export_results_internal(results: Vec<HostInfo>, format: String) -> Result<String, String> {
    match format.as_str() {
        "json" => serde_json::to_string_pretty(&results)
            .map_err(|e| format!("JSON serialization failed: {}", e)),
        "csv" => {
            let mut csv = String::from("IP,Hostname,Port,Service\n");
            for host in &results {
                let hostname = host.hostname.as_deref().unwrap_or("");
                if host.ports.is_empty() {
                    csv.push_str(&format!("{},{},–,–\n", host.ip, hostname));
                } else {
                    for port in &host.ports {
                        csv.push_str(&format!(
                            "{},{},{},{}\n",
                            host.ip, hostname, port.port, port.service_label
                        ));
                    }
                }
            }
            Ok(csv)
        }
        _ => Err(format!(
            "Unsupported format: {}. Use 'json' or 'csv'.",
            format
        )),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;
