//! ServiceType 映射测试
//!
//! 测试目标：验证 ServiceType 从端口号的正确映射和标签生成
//! 覆盖率目标：80%+

use crate::ServiceType;

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.1: ServiceType::from_port 端口映射测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_service_type_from_port_well_known_ports() {
    // Well-known ports (0-1023)
    assert!(matches!(ServiceType::from_port(21), ServiceType::FTP));
    assert!(matches!(ServiceType::from_port(22), ServiceType::SSH));
    assert!(matches!(ServiceType::from_port(23), ServiceType::Telnet));
    assert!(matches!(ServiceType::from_port(25), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(53), ServiceType::DNS));
    assert!(matches!(ServiceType::from_port(67), ServiceType::DHCP));
    assert!(matches!(ServiceType::from_port(68), ServiceType::DHCP));
    assert!(matches!(ServiceType::from_port(80), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(443), ServiceType::HTTPS));
    assert!(matches!(ServiceType::from_port(445), ServiceType::SMB));
}

#[test]
fn test_service_type_from_port_registered_ports() {
    // Registered ports (1024-49151)
    assert!(matches!(ServiceType::from_port(1883), ServiceType::MQTT));
    assert!(matches!(ServiceType::from_port(2375), ServiceType::Docker));
    assert!(matches!(ServiceType::from_port(2376), ServiceType::Docker));
    assert!(matches!(ServiceType::from_port(3000), ServiceType::Gitea));
    assert!(matches!(ServiceType::from_port(3306), ServiceType::MySQL));
    assert!(matches!(ServiceType::from_port(3389), ServiceType::RDP));
    assert!(matches!(ServiceType::from_port(5000), ServiceType::HTTP));
    assert!(matches!(
        ServiceType::from_port(5432),
        ServiceType::PostgreSQL
    ));
    assert!(matches!(ServiceType::from_port(5900), ServiceType::VNC));
    assert!(matches!(ServiceType::from_port(5901), ServiceType::VNC));
    assert!(matches!(ServiceType::from_port(6379), ServiceType::Redis));
    assert!(matches!(
        ServiceType::from_port(6443),
        ServiceType::Kubernetes
    ));
    assert!(matches!(ServiceType::from_port(8080), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(8443), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(8888), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(5173), ServiceType::HTTP));
    assert!(matches!(ServiceType::from_port(9000), ServiceType::MinIO));
    assert!(matches!(
        ServiceType::from_port(9090),
        ServiceType::Prometheus
    ));
    assert!(matches!(
        ServiceType::from_port(9200),
        ServiceType::Elasticsearch
    ));
    assert!(matches!(
        ServiceType::from_port(9300),
        ServiceType::Elasticsearch
    ));
    assert!(matches!(
        ServiceType::from_port(27017),
        ServiceType::MongoDB
    ));
}

#[test]
fn test_service_type_from_port_alternative_smtp_ports() {
    // SMTP 有多个常用端口
    assert!(matches!(ServiceType::from_port(25), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(587), ServiceType::SMTP));
    assert!(matches!(ServiceType::from_port(465), ServiceType::SMTP));
}

#[test]
fn test_service_type_from_port_mqtt_ports() {
    // MQTT 标准端口和 TLS 端口
    assert!(matches!(ServiceType::from_port(1883), ServiceType::MQTT));
    assert!(matches!(ServiceType::from_port(8883), ServiceType::MQTT));
}

#[test]
fn test_service_type_from_port_unknown() {
    // 未知端口应返回 Unknown
    assert!(matches!(ServiceType::from_port(0), ServiceType::Unknown));
    assert!(matches!(ServiceType::from_port(1), ServiceType::Unknown));
    assert!(matches!(ServiceType::from_port(1024), ServiceType::Unknown));
    assert!(matches!(ServiceType::from_port(9999), ServiceType::Unknown));
    assert!(matches!(
        ServiceType::from_port(49152),
        ServiceType::Unknown
    ));
    assert!(matches!(
        ServiceType::from_port(65535),
        ServiceType::Unknown
    ));
}

#[test]
fn test_service_type_from_port_boundary_cases() {
    // 边界测试：u16 最大值
    assert!(matches!(
        ServiceType::from_port(u16::MAX),
        ServiceType::Unknown
    ));
    assert!(matches!(
        ServiceType::from_port(65535),
        ServiceType::Unknown
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.1: ServiceType::label 标签生成测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_service_type_label_basic_services() {
    assert_eq!(ServiceType::SSH.label(), "SSH");
    assert_eq!(ServiceType::HTTP.label(), "HTTP");
    assert_eq!(ServiceType::HTTPS.label(), "HTTPS");
    assert_eq!(ServiceType::FTP.label(), "FTP");
    assert_eq!(ServiceType::Telnet.label(), "Telnet");
    assert_eq!(ServiceType::SMTP.label(), "SMTP");
    assert_eq!(ServiceType::DNS.label(), "DNS");
    assert_eq!(ServiceType::DHCP.label(), "DHCP");
}

#[test]
fn test_service_type_label_database_services() {
    assert_eq!(ServiceType::Redis.label(), "Redis");
    assert_eq!(ServiceType::PostgreSQL.label(), "PostgreSQL");
    assert_eq!(ServiceType::MySQL.label(), "MySQL");
    assert_eq!(ServiceType::MongoDB.label(), "MongoDB");
}

#[test]
fn test_service_type_label_remote_access() {
    assert_eq!(ServiceType::RDP.label(), "RDP");
    assert_eq!(ServiceType::VNC.label(), "VNC");
    assert_eq!(ServiceType::SMB.label(), "SMB");
}

#[test]
fn test_service_type_label_iot_and_messaging() {
    assert_eq!(ServiceType::MQTT.label(), "MQTT");
}

#[test]
fn test_service_type_label_container_orchestration() {
    assert_eq!(ServiceType::Docker.label(), "Docker");
    assert_eq!(ServiceType::Kubernetes.label(), "K8s API");
}

#[test]
fn test_service_type_label_monitoring_and_storage() {
    assert_eq!(ServiceType::Elasticsearch.label(), "Elasticsearch");
    assert_eq!(ServiceType::Grafana.label(), "Grafana");
    assert_eq!(ServiceType::Prometheus.label(), "Prometheus");
    assert_eq!(ServiceType::MinIO.label(), "MinIO");
    assert_eq!(ServiceType::Gitea.label(), "Gitea");
}

#[test]
fn test_service_type_label_unknown() {
    assert_eq!(ServiceType::Unknown.label(), "Unknown");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.1: 端到端映射一致性测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_service_type_port_to_label_consistency() {
    // 验证端口映射到 ServiceType 后再获取标签的一致性
    let test_cases = vec![
        (21, "FTP"),
        (22, "SSH"),
        (23, "Telnet"),
        (25, "SMTP"),
        (53, "DNS"),
        (80, "HTTP"),
        (443, "HTTPS"),
        (3306, "MySQL"),
        (5432, "PostgreSQL"),
        (6379, "Redis"),
        (27017, "MongoDB"),
        (8080, "HTTP"),
    ];

    for (port, expected_label) in test_cases {
        let service = ServiceType::from_port(port);
        let label = service.label();
        assert_eq!(
            label, expected_label,
            "Port {} should map to label '{}'",
            port, expected_label
        );
    }
}

#[test]
fn test_service_type_all_variants_have_labels() {
    // 确保所有 ServiceType 变体都有非空标签
    let all_services = vec![
        ServiceType::SSH,
        ServiceType::HTTP,
        ServiceType::HTTPS,
        ServiceType::FTP,
        ServiceType::Telnet,
        ServiceType::SMTP,
        ServiceType::DNS,
        ServiceType::DHCP,
        ServiceType::Redis,
        ServiceType::PostgreSQL,
        ServiceType::MySQL,
        ServiceType::MongoDB,
        ServiceType::MQTT,
        ServiceType::SMB,
        ServiceType::RDP,
        ServiceType::VNC,
        ServiceType::Docker,
        ServiceType::Kubernetes,
        ServiceType::Elasticsearch,
        ServiceType::Grafana,
        ServiceType::Prometheus,
        ServiceType::MinIO,
        ServiceType::Gitea,
        ServiceType::Unknown,
    ];

    for service in all_services {
        let label = service.label();
        assert!(
            !label.is_empty(),
            "ServiceType {:?} should have a non-empty label",
            service
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.1: 序列化和反序列化测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_service_type_serde_serialization() {
    use serde_json;

    let service = ServiceType::SSH;
    let json = serde_json::to_string(&service).expect("Should serialize");
    assert_eq!(json, "\"SSH\"");

    let service = ServiceType::HTTP;
    let json = serde_json::to_string(&service).expect("Should serialize");
    assert_eq!(json, "\"HTTP\"");

    let service = ServiceType::Unknown;
    let json = serde_json::to_string(&service).expect("Should serialize");
    assert_eq!(json, "\"Unknown\"");
}

#[test]
fn test_service_type_serde_deserialization() {
    use serde_json;

    let service: ServiceType = serde_json::from_str("\"SSH\"").expect("Should deserialize");
    assert!(matches!(service, ServiceType::SSH));

    let service: ServiceType = serde_json::from_str("\"HTTP\"").expect("Should deserialize");
    assert!(matches!(service, ServiceType::HTTP));

    let service: ServiceType = serde_json::from_str("\"Unknown\"").expect("Should deserialize");
    assert!(matches!(service, ServiceType::Unknown));
}

#[test]
fn test_service_type_serde_roundtrip() {
    use serde_json;

    let all_services = vec![
        ServiceType::SSH,
        ServiceType::HTTP,
        ServiceType::HTTPS,
        ServiceType::FTP,
        ServiceType::Telnet,
        ServiceType::SMTP,
        ServiceType::DNS,
        ServiceType::DHCP,
        ServiceType::Redis,
        ServiceType::PostgreSQL,
        ServiceType::MySQL,
        ServiceType::MongoDB,
        ServiceType::MQTT,
        ServiceType::SMB,
        ServiceType::RDP,
        ServiceType::VNC,
        ServiceType::Docker,
        ServiceType::Kubernetes,
        ServiceType::Elasticsearch,
        ServiceType::Grafana,
        ServiceType::Prometheus,
        ServiceType::MinIO,
        ServiceType::Gitea,
        ServiceType::Unknown,
    ];

    for original in all_services {
        let json = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: ServiceType = serde_json::from_str(&json).expect("Should deserialize");

        // 使用 debug 比较，因为 ServiceType 没有实现 PartialEq
        assert_eq!(
            format!("{:?}", original),
            format!("{:?}", deserialized),
            "Roundtrip failed for {:?}",
            original
        );
    }
}
