use super::*;

#[test]
fn hello_without_protocol_reads_as_zero() {
    // A client older than the field is refused, not misread.
    let p: ShellClientPacket =
        serde_json::from_str(r#"{"kind":"hello","user":"Admin","width":80,"height":24}"#).unwrap();
    assert!(matches!(p, ShellClientPacket::Hello { protocol: 0, .. }));
    let p: ShellClientPacket = serde_json::from_str(r#"{"kind":"monitor_hello"}"#).unwrap();
    assert!(matches!(p, ShellClientPacket::MonitorHello { protocol: 0 }));
}

#[test]
fn status_roundtrips_with_the_frontend_field_names() {
    let status = Status {
        user: "LD".into(),
        mode: "KBD".into(),
        walk: Some([2, 5]),
        claims: vec![ClaimStatus {
            object: "Group 1".into(),
            mine: true,
            owner: "LD".into(),
            editor: false,
        }],
        ..Status::default()
    };
    let json = serde_json::to_value(ShellServerPacket::Status {
        status: Box::new(status.clone()),
    })
    .unwrap();
    assert_eq!(json["kind"], "status");
    assert_eq!(json["status"]["walk"], serde_json::json!([2, 5]));
    assert_eq!(json["status"]["show"]["startup"]["mode"], "last");
    assert_eq!(
        json["status"]["lumasession"]["decision"],
        serde_json::Value::Null
    );
    let back: ShellServerPacket = serde_json::from_value(json).unwrap();
    assert!(matches!(back, ShellServerPacket::Status { status: s } if *s == status));
}

#[test]
fn mismatch_message_names_the_code() {
    assert!(client_mismatch(1, 2).contains("[E-WIRE-001]"));
}

#[test]
fn a_beacon_is_leaving_only_when_it_says_so() {
    let mut beacon: discovery::SessionDiscoveryWire = serde_json::from_value(serde_json::json!({
        "station_name": "desk1", "authority": "", "cmd_port": 47653, "shell_port": 47654,
        "session_count": 0, "active_shell_clients": 0, "build_id": "", "sent_unix_ms": 0
    }))
    .unwrap();
    assert!(!beacon.leaving, "a beacon without the field is a running station");
    assert!(serde_json::to_value(&beacon).unwrap().get("leaving").is_none());
    beacon.leaving = true;
    let json = serde_json::to_value(&beacon).unwrap();
    assert_eq!(json["leaving"], true);
    let back: discovery::SessionDiscoveryWire = serde_json::from_value(json).unwrap();
    assert!(back.leaving);
}
