mod common;

use common::{structured, text};
use toolkit::CidrNetworkTool;

#[test]
fn cidr_network_class_c() {
    let result = CidrNetworkTool {
        ip: "192.168.1.10".to_string(),
        mask: "255.255.255.0".to_string(),
    }
    .call_tool()
    .expect("cidr_network");

    let value = structured(&result);
    assert_eq!(value["ip"].as_str(), Some("192.168.1.10"));
    assert_eq!(value["mask"].as_str(), Some("255.255.255.0"));
    assert_eq!(value["prefix_length"].as_u64(), Some(24));
    assert_eq!(value["network"].as_str(), Some("192.168.1.0"));
    assert_eq!(value["broadcast"].as_str(), Some("192.168.1.255"));
    assert_eq!(value["first_host"].as_str(), Some("192.168.1.1"));
    assert_eq!(value["last_host"].as_str(), Some("192.168.1.254"));
    assert_eq!(value["address_count"].as_u64(), Some(256));
    assert_eq!(value["usable_hosts"].as_u64(), Some(254));
    assert_eq!(value["cidr"].as_str(), Some("192.168.1.0/24"));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn cidr_network_prefix_and_special_cases() {
    let class_a = CidrNetworkTool {
        ip: "10.0.0.5".to_string(),
        mask: "/8".to_string(),
    }
    .call_tool()
    .expect("cidr_network");
    let value = structured(&class_a);
    assert_eq!(value["prefix_length"].as_u64(), Some(8));
    assert_eq!(value["network"].as_str(), Some("10.0.0.0"));
    assert_eq!(value["broadcast"].as_str(), Some("10.255.255.255"));
    assert_eq!(value["usable_hosts"].as_u64(), Some(16_777_214));

    let point_to_point = CidrNetworkTool {
        ip: "192.168.1.0".to_string(),
        mask: "31".to_string(),
    }
    .call_tool()
    .expect("cidr_network");
    let value = structured(&point_to_point);
    assert_eq!(value["network"].as_str(), Some("192.168.1.0"));
    assert_eq!(value["broadcast"].as_str(), Some("192.168.1.1"));
    assert_eq!(value["first_host"].as_str(), Some("192.168.1.0"));
    assert_eq!(value["last_host"].as_str(), Some("192.168.1.1"));
    assert_eq!(value["address_count"].as_u64(), Some(2));
    assert_eq!(value["usable_hosts"].as_u64(), Some(2));

    let host = CidrNetworkTool {
        ip: "192.168.1.7".to_string(),
        mask: "255.255.255.255".to_string(),
    }
    .call_tool()
    .expect("cidr_network");
    let value = structured(&host);
    assert_eq!(value["network"].as_str(), Some("192.168.1.7"));
    assert_eq!(value["broadcast"].as_str(), Some("192.168.1.7"));
    assert_eq!(value["address_count"].as_u64(), Some(1));
    assert_eq!(value["usable_hosts"].as_u64(), Some(1));

    let default_route = CidrNetworkTool {
        ip: "1.2.3.4".to_string(),
        mask: "0".to_string(),
    }
    .call_tool()
    .expect("cidr_network");
    let value = structured(&default_route);
    assert_eq!(value["network"].as_str(), Some("0.0.0.0"));
    assert_eq!(value["broadcast"].as_str(), Some("255.255.255.255"));
    assert_eq!(value["first_host"].as_str(), Some("0.0.0.1"));
    assert_eq!(value["last_host"].as_str(), Some("255.255.255.254"));
    assert_eq!(value["address_count"].as_u64(), Some(4_294_967_296));
    assert_eq!(value["usable_hosts"].as_u64(), Some(4_294_967_294));
}

#[test]
fn cidr_network_rejects_invalid_input() {
    assert!(
        CidrNetworkTool {
            ip: "192.168.1.999".to_string(),
            mask: "24".to_string(),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CidrNetworkTool {
            ip: "::1".to_string(),
            mask: "24".to_string(),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CidrNetworkTool {
            ip: "192.168.1.1".to_string(),
            mask: "33".to_string(),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CidrNetworkTool {
            ip: "192.168.1.1".to_string(),
            mask: "255.0.255.0".to_string(),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CidrNetworkTool {
            ip: "192.168.1.1".to_string(),
            mask: "abc".to_string(),
        }
        .call_tool()
        .is_err()
    );
}
