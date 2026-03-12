//! ARP 扫描功能测试
//!
//! 测试目标：验证 ARP 扫描功能正确解析系统输出
//! 覆盖率目标：80%+

use crate::{
    active_arp_scan, arp_scan, parse_arp_output, parse_arping_output, parse_ip_neigh_output,
    parse_proc_net_arp, ArpEntry,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1: ARP 输出解析测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_arp_output_macos_format() {
    // macOS 格式: ? (192.168.1.1) at ab:cd:ef:12:34:56 on en0 [ethernet]
    let output = r#"? (192.168.1.1) at ab:cd:ef:12:34:56 on en0 ifscope [ethernet]
? (192.168.1.100) at 12:34:56:78:9a:bc on en0 ifscope [ethernet]
? (192.168.1.254) at fe:dc:ba:98:76:54 on en0 ifscope permanent [ethernet]"#;

    let entries = parse_arp_output(output, "192.168.1");

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].ip, "192.168.1.1");
    assert_eq!(entries[0].mac, "ab:cd:ef:12:34:56");
    assert_eq!(entries[0].interface, Some("en0".to_string()));

    assert_eq!(entries[1].ip, "192.168.1.100");
    assert_eq!(entries[1].mac, "12:34:56:78:9a:bc");

    assert_eq!(entries[2].ip, "192.168.1.254");
    assert_eq!(entries[2].mac, "fe:dc:ba:98:76:54");
}

#[test]
fn test_parse_arp_output_linux_format() {
    // Linux 格式: ? (192.168.1.1) at ab:cd:ef:12:34:56 [ether] on eth0
    let output = r#"? (192.168.1.1) at ab:cd:ef:12:34:56 [ether] on eth0
? (192.168.1.50) at 11:22:33:44:55:66 [ether] on eth0
? (10.0.0.1) at aa:bb:cc:dd:ee:ff [ether] on eth1"#;

    let entries = parse_arp_output(output, "192.168.1");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].ip, "192.168.1.1");
    assert_eq!(entries[0].mac, "ab:cd:ef:12:34:56");
    assert_eq!(entries[0].interface, Some("eth0".to_string()));

    assert_eq!(entries[1].ip, "192.168.1.50");
    assert_eq!(entries[1].mac, "11:22:33:44:55:66");
}

#[test]
fn test_parse_arp_output_different_subnet() {
    // 测试只返回指定子网的条目
    let output = r#"? (192.168.1.1) at ab:cd:ef:12:34:56 on en0
? (192.168.2.1) at 11:22:33:44:55:66 on en0
? (10.0.0.1) at aa:bb:cc:dd:ee:ff on en0"#;

    let entries = parse_arp_output(output, "192.168.1");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ip, "192.168.1.1");
}

#[test]
fn test_parse_arp_output_empty() {
    let entries = parse_arp_output("", "192.168.1");
    assert!(entries.is_empty());
}

#[test]
fn test_parse_arp_output_invalid_mac() {
    // 测试无效 MAC 地址被过滤
    let output = r#"? (192.168.1.1) at (incomplete) on en0
? (192.168.1.2) at ab:cd:ef:12:34:56 on en0"#;

    let entries = parse_arp_output(output, "192.168.1");

    // 只有有效的 MAC 地址会被解析
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ip, "192.168.1.2");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 2: ip neigh 输出解析测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_ip_neigh_output() {
    // Linux ip neigh 格式
    let output = r#"192.168.1.1 dev eth0 lladdr ab:cd:ef:12:34:56 REACHABLE
192.168.1.100 dev eth0 lladdr 12:34:56:78:9a:bc STALE
192.168.1.200 dev eth0 lladdr aa:bb:cc:dd:ee:ff DELAY
10.0.0.1 dev eth1 lladdr 11:22:33:44:55:66 REACHABLE"#;

    let entries = parse_ip_neigh_output(output, "192.168.1");

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].ip, "192.168.1.1");
    assert_eq!(entries[0].mac, "ab:cd:ef:12:34:56");
    assert_eq!(entries[0].interface, Some("eth0".to_string()));

    assert_eq!(entries[1].ip, "192.168.1.100");
    assert_eq!(entries[1].mac, "12:34:56:78:9a:bc");

    assert_eq!(entries[2].ip, "192.168.1.200");
    assert_eq!(entries[2].mac, "aa:bb:cc:dd:ee:ff");
}

#[test]
fn test_parse_ip_neigh_output_empty() {
    let entries = parse_ip_neigh_output("", "192.168.1");
    assert!(entries.is_empty());
}

#[test]
fn test_parse_ip_neigh_output_malformed() {
    // 测试格式不正确的行被跳过
    let output = r#"192.168.1.1 dev eth0 lladdr ab:cd:ef:12:34:56 REACHABLE
malformed line
192.168.1.2 dev eth0"#;

    let entries = parse_ip_neigh_output(output, "192.168.1");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ip, "192.168.1.1");
}

#[test]
fn test_parse_proc_net_arp() {
    let output = r#"IP address       HW type     Flags       HW address            Mask     Device
192.168.1.1       0x1         0x2         ab:cd:ef:12:34:56     *        eth0
192.168.1.50      0x1         0x2         11:22:33:44:55:66     *        wlan0
10.0.0.1          0x1         0x2         aa:bb:cc:dd:ee:ff     *        eth1"#;

    let entries = parse_proc_net_arp(output, "192.168.1");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].ip, "192.168.1.1");
    assert_eq!(entries[0].mac, "ab:cd:ef:12:34:56");
    assert_eq!(entries[0].interface, Some("eth0".to_string()));
    assert_eq!(entries[1].interface, Some("wlan0".to_string()));
}

#[test]
fn test_parse_proc_net_arp_skips_incomplete_entries() {
    let output = r#"IP address       HW type     Flags       HW address            Mask     Device
192.168.1.1       0x1         0x0         00:00:00:00:00:00     *        eth0
192.168.1.2       0x1         0x2         (incomplete)          *        eth0
192.168.1.3       0x1         0x2         aa:bb:cc:dd:ee:ff     *        eth0"#;

    let entries = parse_proc_net_arp(output, "192.168.1");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].ip, "192.168.1.3");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3: arping 输出解析测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_arping_output() {
    // 典型 arping 输出
    let output = "64 bytes from 192.168.1.1 (ab:cd:ef:12:34:56): icmp_seq=0 time=1.234 ms";

    let mac = parse_arping_output(output);

    assert_eq!(mac, Some("ab:cd:ef:12:34:56".to_string()));
}

#[test]
fn test_parse_arping_output_multiple_lines() {
    let output = r#"ARPING 192.168.1.1 from 192.168.1.100 eth0
64 bytes from 192.168.1.1 (ab:cd:ef:12:34:56): icmp_seq=0 time=1.234 ms
64 bytes from 192.168.1.1 (ab:cd:ef:12:34:56): icmp_seq=1 time=0.987 ms"#;

    let mac = parse_arping_output(output);

    assert_eq!(mac, Some("ab:cd:ef:12:34:56".to_string()));
}

#[test]
fn test_parse_arping_output_no_mac() {
    let output = "Timeout";
    let mac = parse_arping_output(output);
    assert_eq!(mac, None);
}

#[test]
fn test_parse_arping_output_empty() {
    let mac = parse_arping_output("");
    assert_eq!(mac, None);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 4: ARP Entry 结构体测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_arp_entry_creation() {
    let entry = ArpEntry {
        ip: "192.168.1.1".to_string(),
        mac: "ab:cd:ef:12:34:56".to_string(),
        interface: Some("eth0".to_string()),
    };

    assert_eq!(entry.ip, "192.168.1.1");
    assert_eq!(entry.mac, "ab:cd:ef:12:34:56");
    assert_eq!(entry.interface, Some("eth0".to_string()));
}

#[test]
fn test_arp_entry_serialization() {
    let entry = ArpEntry {
        ip: "192.168.1.100".to_string(),
        mac: "11:22:33:44:55:66".to_string(),
        interface: None,
    };

    let json = serde_json::to_string(&entry).expect("Should serialize");
    assert!(json.contains("192.168.1.100"));
    assert!(json.contains("11:22:33:44:55:66"));

    let deserialized: ArpEntry = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.ip, "192.168.1.100");
    assert_eq!(deserialized.mac, "11:22:33:44:55:66");
    assert_eq!(deserialized.interface, None);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5: 集成测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_arp_scan_does_not_panic() {
    // 这个测试验证 arp_scan 函数不会 panic
    // 实际结果取决于系统环境
    let _entries = arp_scan("192.168.1");
    // 函数应该成功返回，即使没有找到设备
}

#[test]
fn test_mac_address_format_variations() {
    // 测试不同格式的 MAC 地址
    let test_cases = vec![
        ("ab:cd:ef:12:34:56", true),
        ("AB:CD:EF:12:34:56", true),
        ("ab-cd-ef-12-34-56", false), // 我们的解析器使用冒号分隔
        ("abcdef123456", false),      // 没有分隔符
        ("incomplete", false),        // 无效值
    ];

    for (mac, should_be_valid) in test_cases {
        // 检查 MAC 地址格式
        let is_valid = mac.len() == 17 && mac.contains(':');
        assert_eq!(
            is_valid, should_be_valid,
            "MAC address {} validation failed",
            mac
        );
    }
}

#[test]
fn test_subnet_filtering() {
    // 测试子网过滤逻辑
    let entries = vec![
        ArpEntry {
            ip: "192.168.1.1".to_string(),
            mac: "aa:bb:cc:dd:ee:ff".to_string(),
            interface: None,
        },
        ArpEntry {
            ip: "192.168.2.1".to_string(),
            mac: "11:22:33:44:55:66".to_string(),
            interface: None,
        },
        ArpEntry {
            ip: "10.0.0.1".to_string(),
            mac: "aa:aa:aa:aa:aa:aa".to_string(),
            interface: None,
        },
    ];

    let filtered: Vec<_> = entries
        .into_iter()
        .filter(|e| e.ip.starts_with("192.168.1"))
        .collect();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].ip, "192.168.1.1");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 6: 主动 ARP 扫描测试
// ═══════════════════════════════════════════════════════════════════════════════

/// 测试主动 ARP 扫描不会 panic
#[tokio::test]
async fn test_active_arp_scan_does_not_panic() {
    // 使用一个不常见的子网进行测试
    let entries = active_arp_scan("10.255.255").await;
    // 应该成功返回（即使找不到设备）
    println!("Active ARP scan found {} entries", entries.len());
}

/// 测试主动 ARP 扫描本机回环地址
#[tokio::test]
async fn test_active_arp_scan_localhost_subnet() {
    // 扫描 localhost 子网，应该快速完成
    let entries = active_arp_scan("127.0.0").await;
    // localhost 不会有 ARP 响应，但函数应该正常返回
    println!("Active ARP scan on 127.0.0 found {} entries", entries.len());
}

/// 测试主动 ARP 扫描并发性能
#[tokio::test]
async fn test_active_arp_scan_concurrent() {
    use tokio::time::{timeout, Duration};

    // 主动 ARP 扫描应该并发执行，快速返回
    let start = std::time::Instant::now();
    let result = timeout(
        Duration::from_secs(30),    // 设置一个合理的超时
        active_arp_scan("192.0.2"), // 使用 TEST-NET-1 子网进行测试
    )
    .await;

    let elapsed = start.elapsed();

    // 即使扫描完成，也应该在合理时间内返回
    // 扫描 254 个 IP，每个大约 500ms 超时，但并发执行
    // 正常情况下应该少于 30 秒
    assert!(
        elapsed < Duration::from_secs(60),
        "Active ARP scan took too long: {:?}",
        elapsed
    );

    println!(
        "Active ARP scan completed in {:?}, result: {:?}",
        elapsed, result
    );
}
