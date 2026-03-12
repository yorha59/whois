//! Unit tests for WhoIs scanner backend

pub mod service_type_test;
pub mod export_test;
pub mod network_test;

use crate::{export_results_internal as export_results, HostInfo, PortInfo, ServiceType};

/// Test port to service type mapping
#[test]
fn test_service_type_from_port() {
    // Test well-known ports
    assert!(matches!(ServiceType::from_port(22), ServiceType::SSH));
    assert!(matches!(ServiceType::from_port(80), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(443), ServiceType::HTTPS));
    assert!(matches!(ServiceType::from_port(21), ServiceType::FTP));
    assert!(matches!(ServiceType::from_port(23), ServiceType::Telnet));
    assert!(matches!(ServiceType::from_port(25), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(53), ServiceType::DNS));
    assert!(matches!(ServiceType::from_port(3306), ServiceType::MySQL));
    assert!(matches!(ServiceType::from_port(5432), ServiceType::PostgreSQL));
    assert!(matches!(ServiceType::from_port(6379), ServiceType::Redis));
    assert!(matches!(ServiceType::from_port(27017), ServiceType::MongoDB));
    assert!(matches!(ServiceType::from_port(3389), ServiceType::RDP));
    assert!(matches!(ServiceType::from_port(445), ServiceType::SMB));

    // Test alternative ports
    assert!(matches!(ServiceType::from_port(587), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(465), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(8080), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(8443), ServiceType::HTTP)); // HTTP alternate
    assert!(matches!(ServiceType::from_port(5900), ServiceType::VNC));
    assert!(matches!(ServiceType::from_port(5901), ServiceType::VNC));

    // Test unknown port
    assert!(matches!(ServiceType::from_port(9999), ServiceType::Unknown));
    assert!(matches!(ServiceType::from_port(0), ServiceType::Unknown));
}

/// Test service type label
#[test]
fn test_service_type_label() {
    assert_eq!(ServiceType::SSH.label(), "SSH");
    assert_eq!(ServiceType::HTTP.label(), "HTTP");
    assert_eq!(ServiceType::HTTPS.label(), "HTTPS");
    assert_eq!(ServiceType::FTP.label(), "FTP");
    assert_eq!(ServiceType::Telnet.label(), "Telnet");
    assert_eq!(ServiceType::SMTP.label(), "SMTP");
    assert_eq!(ServiceType::DNS.label(), "DNS");
    assert_eq!(ServiceType::DHCP.label(), "DHCP");
    assert_eq!(ServiceType::Redis.label(), "Redis");
    assert_eq!(ServiceType::PostgreSQL.label(), "PostgreSQL");
    assert_eq!(ServiceType::MySQL.label(), "MySQL");
    assert_eq!(ServiceType::MongoDB.label(), "MongoDB");
    assert_eq!(ServiceType::MQTT.label(), "MQTT");
    assert_eq!(ServiceType::SMB.label(), "SMB");
    assert_eq!(ServiceType::RDP.label(), "RDP");
    assert_eq!(ServiceType::VNC.label(), "VNC");
    assert_eq!(ServiceType::Docker.label(), "Docker");
    assert_eq!(ServiceType::Kubernetes.label(), "K8s API");
    assert_eq!(ServiceType::Elasticsearch.label(), "Elasticsearch");
    assert_eq!(ServiceType::Grafana.label(), "Grafana");
    assert_eq!(ServiceType::Prometheus.label(), "Prometheus");
    assert_eq!(ServiceType::MinIO.label(), "MinIO");
    assert_eq!(ServiceType::Gitea.label(), "Gitea");
    assert_eq!(ServiceType::Unknown.label(), "Unknown");
}

/// Test JSON export functionality
#[test]
fn test_export_results_json() {
    let results = vec![
        HostInfo {
            ip: "192.168.1.1".to_string(),
            hostname: Some("router.local".to_string()),
            ports: vec![
                PortInfo {
                    port: 80,
                    service: ServiceType::HTTP,
                    service_label: "HTTP".to_string(),
                },
                PortInfo {
                    port: 443,
                    service: ServiceType::HTTPS,
                    service_label: "HTTPS".to_string(),
                },
            ],
        },
        HostInfo {
            ip: "192.168.1.100".to_string(),
            hostname: None,
            ports: vec![PortInfo {
                port: 22,
                service: ServiceType::SSH,
                service_label: "SSH".to_string(),
            }],
        },
    ];

    let json_result = export_results(results.clone(), "json".to_string());
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.contains("192.168.1.1"));
    assert!(json_str.contains("router.local"));
    assert!(json_str.contains("192.168.1.100"));
    assert!(json_str.contains("80"));
    assert!(json_str.contains("443"));
    assert!(json_str.contains("22"));
    assert!(json_str.contains("HTTP"));
    assert!(json_str.contains("HTTPS"));
    assert!(json_str.contains("SSH"));
}

/// Test CSV export functionality
#[test]
fn test_export_results_csv() {
    let results = vec![
        HostInfo {
            ip: "192.168.1.1".to_string(),
            hostname: Some("router.local".to_string()),
            ports: vec![
                PortInfo {
                    port: 80,
                    service: ServiceType::HTTP,
                    service_label: "HTTP".to_string(),
                },
                PortInfo {
                    port: 443,
                    service: ServiceType::HTTPS,
                    service_label: "HTTPS".to_string(),
                },
            ],
        },
        HostInfo {
            ip: "192.168.1.100".to_string(),
            hostname: None,
            ports: vec![PortInfo {
                port: 22,
                service: ServiceType::SSH,
                service_label: "SSH".to_string(),
            }],
        },
    ];

    let csv_result = export_results(results, "csv".to_string());
    assert!(csv_result.is_ok());

    let csv_str = csv_result.unwrap();
    // Check header
    assert!(csv_str.starts_with("IP,Hostname,Port,Service\n"));
    // Check data rows
    assert!(csv_str.contains("192.168.1.1"));
    assert!(csv_str.contains("router.local"));
    assert!(csv_str.contains("192.168.1.100"));
    assert!(csv_str.contains(",80,HTTP"));
    assert!(csv_str.contains(",443,HTTPS"));
    assert!(csv_str.contains(",22,SSH"));
}

/// Test CSV export with empty ports
#[test]
fn test_export_results_csv_empty_ports() {
    let results = vec![HostInfo {
        ip: "192.168.1.1".to_string(),
        hostname: None,
        ports: vec![],
    }];

    let csv_result = export_results(results, "csv".to_string());
    assert!(csv_result.is_ok());

    let csv_str = csv_result.unwrap();
    assert!(csv_str.contains("192.168.1.1,,–,–"));
}

/// Test unsupported format
#[test]
fn test_export_results_unsupported_format() {
    let results: Vec<HostInfo> = vec![];
    let result = export_results(results, "xml".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported format"));
}

/// Test network subnet detection parsing
#[test]
fn test_detect_network_subnet() {
    // Test that subnet extraction logic works correctly
    // This simulates the parsing logic from detect_network()

    fn extract_subnet(ip: &str) -> Option<String> {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            Some(format!("{}.{}.{}" ,parts[0], parts[1], parts[2]))
        } else {
            None
        }
    }

    // Test valid IPs
    assert_eq!(
        extract_subnet("192.168.1.100"),
        Some("192.168.1".to_string())
    );
    assert_eq!(
        extract_subnet("10.0.0.1"),
        Some("10.0.0".to_string())
    );
    assert_eq!(
        extract_subnet("172.16.255.254"),
        Some("172.16.255".to_string())
    );

    // Test edge cases
    assert_eq!(
        extract_subnet("0.0.0.0"),
        Some("0.0.0".to_string())
    );
    assert_eq!(
        extract_subnet("255.255.255.255"),
        Some("255.255.255".to_string())
    );

    // Test invalid IPs
    assert_eq!(extract_subnet("invalid"), None);
    assert_eq!(extract_subnet("192.168.1"), None);
    assert_eq!(extract_subnet("192.168.1.1.1"), None);
    assert_eq!(extract_subnet(""), None);
}

/// Test HostInfo and PortInfo struct creation
#[test]
fn test_struct_creation() {
    let port = PortInfo {
        port: 8080,
        service: ServiceType::HTTP,
        service_label: "HTTP".to_string(),
    };
    assert_eq!(port.port, 8080);
    assert!(matches!(port.service, ServiceType::HTTP));
    assert_eq!(port.service_label, "HTTP");

    let host = HostInfo {
        ip: "10.0.0.1".to_string(),
        hostname: Some("testhost".to_string()),
        ports: vec![port],
    };
    assert_eq!(host.ip, "10.0.0.1");
    assert_eq!(host.hostname, Some("testhost".to_string()));
    assert_eq!(host.ports.len(), 1);
}
