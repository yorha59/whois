//! 导出功能测试
//!
//! 测试目标：验证 scan results 导出为 JSON 和 CSV 格式的正确性
//! 覆盖率目标：80%+

use crate::{export_results_internal as export_results, HostInfo, PortInfo, ServiceType};

// 辅助函数：创建测试数据
fn create_test_port(port: u16, service: ServiceType) -> PortInfo {
    PortInfo {
        port,
        service: service.clone(),
        service_label: service.label().to_string(),
    }
}

fn create_test_host(ip: &str, hostname: Option<&str>, ports: Vec<PortInfo>) -> HostInfo {
    HostInfo {
        ip: ip.to_string(),
        hostname: hostname.map(|h| h.to_string()),
        ports,
        mdns_services: None,
        ssdp_devices: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: JSON 导出测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_export_json_empty_results() {
    let results: Vec<HostInfo> = vec![];
    let result = export_results(results, "json".to_string());

    assert!(result.is_ok());
    let json = result.unwrap();
    assert_eq!(json, "[]");
}

#[test]
fn test_export_json_single_host_single_port() {
    let results = vec![create_test_host(
        "192.168.1.1",
        Some("router.local"),
        vec![create_test_port(22, ServiceType::SSH)],
    )];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("192.168.1.1"));
    assert!(json.contains("router.local"));
    assert!(json.contains("22"));
    assert!(json.contains("SSH"));

    // 验证是合法的 JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[test]
fn test_export_json_single_host_multiple_ports() {
    let results = vec![create_test_host(
        "192.168.1.100",
        Some("server.local"),
        vec![
            create_test_port(22, ServiceType::SSH),
            create_test_port(80, ServiceType::HTTP),
            create_test_port(443, ServiceType::HTTPS),
        ],
    )];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");

    let host = &parsed[0];
    assert_eq!(host["ip"], "192.168.1.100");
    assert_eq!(host["hostname"], "server.local");
    assert!(host["ports"].as_array().unwrap().len() == 3);
}

#[test]
fn test_export_json_multiple_hosts() {
    let results = vec![
        create_test_host(
            "192.168.1.1",
            Some("router"),
            vec![create_test_port(80, ServiceType::HTTP)],
        ),
        create_test_host(
            "192.168.1.100",
            Some("server"),
            vec![
                create_test_port(22, ServiceType::SSH),
                create_test_port(443, ServiceType::HTTPS),
            ],
        ),
        create_test_host("192.168.1.200", None, vec![create_test_port(3306, ServiceType::MySQL)]),
    ];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 3);
}

#[test]
fn test_export_json_host_without_hostname() {
    let results = vec![create_test_host(
        "192.168.1.50",
        None,
        vec![create_test_port(8080, ServiceType::HTTP)],
    )];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert!(parsed[0]["hostname"].is_null());
}

#[test]
fn test_export_json_all_service_types() {
    let results = vec![create_test_host(
        "192.168.1.1",
        None,
        vec![
            create_test_port(21, ServiceType::FTP),
            create_test_port(22, ServiceType::SSH),
            create_test_port(23, ServiceType::Telnet),
            create_test_port(25, ServiceType::SMTP),
            create_test_port(53, ServiceType::DNS),
            create_test_port(80, ServiceType::HTTP),
            create_test_port(443, ServiceType::HTTPS),
            create_test_port(3306, ServiceType::MySQL),
            create_test_port(5432, ServiceType::PostgreSQL),
            create_test_port(6379, ServiceType::Redis),
            create_test_port(27017, ServiceType::MongoDB),
        ],
    )];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    let ports = parsed[0]["ports"].as_array().unwrap();
    assert_eq!(ports.len(), 11);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: CSV 导出测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_export_csv_empty_results() {
    let results: Vec<HostInfo> = vec![];
    let result = export_results(results, "csv".to_string());

    assert!(result.is_ok());
    let csv = result.unwrap();
    assert_eq!(csv, "IP,Hostname,Port,Service\n");
}

#[test]
fn test_export_csv_single_host_single_port() {
    let results = vec![create_test_host(
        "192.168.1.1",
        Some("router.local"),
        vec![create_test_port(22, ServiceType::SSH)],
    )];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2); // 1 header + 1 data row
    assert_eq!(lines[0], "IP,Hostname,Port,Service");
    assert!(lines[1].contains("192.168.1.1"));
    assert!(lines[1].contains("router.local"));
    assert!(lines[1].contains("22"));
    assert!(lines[1].contains("SSH"));
}

#[test]
fn test_export_csv_single_host_multiple_ports() {
    let results = vec![create_test_host(
        "192.168.1.100",
        Some("server"),
        vec![
            create_test_port(22, ServiceType::SSH),
            create_test_port(80, ServiceType::HTTP),
            create_test_port(443, ServiceType::HTTPS),
        ],
    )];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 4); // 1 header + 3 data rows

    // 验证所有端口都被导出
    let data_rows = &lines[1..];
    assert!(data_rows.iter().any(|line| line.contains("22")));
    assert!(data_rows.iter().any(|line| line.contains("80")));
    assert!(data_rows.iter().any(|line| line.contains("443")));
}

#[test]
fn test_export_csv_multiple_hosts() {
    let results = vec![
        create_test_host(
            "192.168.1.1",
            Some("router"),
            vec![create_test_port(80, ServiceType::HTTP)],
        ),
        create_test_host(
            "192.168.1.100",
            Some("server"),
            vec![
                create_test_port(22, ServiceType::SSH),
                create_test_port(443, ServiceType::HTTPS),
            ],
        ),
    ];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 4); // 1 header + 3 data rows
}

#[test]
fn test_export_csv_host_without_hostname() {
    let results = vec![create_test_host(
        "192.168.1.50",
        None,
        vec![create_test_port(22, ServiceType::SSH)],
    )];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    // 没有 hostname 时应该显示空字符串
    assert!(lines[1].contains("192.168.1.50,,22"));
}

#[test]
fn test_export_csv_host_without_ports() {
    // 虽然正常情况下不应该有这种情况，但测试边界情况
    let results = vec![create_test_host("192.168.1.1", Some("router"), vec![])];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    assert!(csv.contains("192.168.1.1,router,–,–"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 错误处理测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_export_unsupported_format() {
    let results = vec![create_test_host(
        "192.168.1.1",
        None,
        vec![create_test_port(80, ServiceType::HTTP)],
    )];

    // 测试不支持的大小写
    let result = export_results(results.clone(), "XML".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported format"));

    // 测试完全不支持的格式
    let result = export_results(results.clone(), "yaml".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported format"));

    // 测试空格式字符串
    let result = export_results(results, "".to_string());
    assert!(result.is_err());
}

#[test]
fn test_export_format_case_sensitivity() {
    let results = vec![create_test_host(
        "192.168.1.1",
        None,
        vec![create_test_port(80, ServiceType::HTTP)],
    )];

    // 测试大写格式（当前实现是大小写敏感的）
    let result = export_results(results.clone(), "JSON".to_string());
    assert!(result.is_err(), "Current implementation is case-sensitive for JSON");

    let result = export_results(results, "CSV".to_string());
    assert!(result.is_err(), "Current implementation is case-sensitive for CSV");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 边界情况测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_export_csv_with_special_characters() {
    // 测试包含特殊字符的 hostname
    let results = vec![create_test_host(
        "192.168.1.1",
        Some("router-with.dots.and-dashes"),
        vec![create_test_port(80, ServiceType::HTTP)],
    )];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    assert!(csv.contains("router-with.dots.and-dashes"));
}

#[test]
fn test_export_large_dataset() {
    // 测试大量数据导出
    let mut results = Vec::new();
    for i in 1..=100 {
        let ports: Vec<PortInfo> = (1..=10)
            .map(|p| create_test_port(p as u16 * 1000, ServiceType::HTTP))
            .collect();
        results.push(create_test_host(
            &format!("192.168.1.{}", i),
            Some(&format!("host{}", i)),
            ports,
        ));
    }

    // JSON 导出
    let json_result = export_results(results.clone(), "json".to_string());
    assert!(json_result.is_ok());

    let json = json_result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 100);

    // CSV 导出
    let csv_result = export_results(results, "csv".to_string());
    assert!(csv_result.is_ok());

    let csv = csv_result.unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    // 1 header + 100 hosts * 10 ports = 1001
    assert_eq!(lines.len(), 1001);
}

#[test]
fn test_export_json_pretty_format() {
    let results = vec![create_test_host(
        "192.168.1.1",
        Some("router"),
        vec![create_test_port(80, ServiceType::HTTP)],
    )];

    let result = export_results(results, "json".to_string());
    assert!(result.is_ok());

    let json = result.unwrap();
    // 验证是 pretty-printed JSON
    assert!(json.contains('\n'), "JSON should be pretty-printed with newlines");
    assert!(json.contains("  "), "JSON should be pretty-printed with indentation");
}

#[test]
fn test_export_csv_unicode_support() {
    // 测试 Unicode 字符支持
    let results = vec![create_test_host(
        "192.168.1.1",
        Some("服务器"), // Chinese characters
        vec![create_test_port(80, ServiceType::HTTP)],
    )];

    let result = export_results(results, "csv".to_string());
    assert!(result.is_ok());

    let csv = result.unwrap();
    assert!(csv.contains("服务器"));
}
